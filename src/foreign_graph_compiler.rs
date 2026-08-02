// r[impl foreign_derivation_import.exact_graph_compilation]
// r[impl foreign_derivation_import.foreign_builtin_lowering]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use bstr::BString;
use crunch_build::EXECUTION_PROFILE_BINDING_ENV;
use crunch_build::ExecutionProfile;
use crunch_build::FETCH_BUILDER;
use crunch_build::FOREIGN_FETCH_CANDIDATES_ENV;
use crunch_build::bind_execution_profile;
use crunch_build::execution_profile_digest;
use crunch_build::foreign_profile_for_producer;
use crunch_build::validate_execution_profile;
use crunch_glue::ResolvedDerivationRequest;
use data_encoding::HEXLOWER;
use nix_compat::derivation::Derivation;
use nix_compat::derivation::Output;
use nix_compat::nixhash::CAHash;
use nix_compat::nixhash::HashAlgo;
use nix_compat::nixhash::NixHash;
use nix_compat::store_path::StorePath;
use nix_compat::store_path::build_text_path_with_store_dir;
use serde::Deserialize;
use serde::Serialize;

use crate::foreign_derivation_import::FixedOutputMetadata;
use crate::foreign_derivation_import::ForeignDerivationGraph;
use crate::foreign_derivation_import::ForeignDerivationNode;
use crate::foreign_derivation_import::ImportDiagnostic;
use crate::foreign_derivation_import::SourcePayload;
use crate::foreign_derivation_import::SourceRef;
use crate::foreign_derivation_import::validate_graph;

const FOREIGN_INPUT_ADDRESSING_MODE: &str = "input-addressed";
const FIXED_OUTPUT_FETCH_BUILTIN: &str = "fixed-output-fetch";
const FOREIGN_DOWNLOAD_BUILTIN: &str = "builtin:download";
const FOREIGN_GIT_DOWNLOAD_BUILTIN: &str = "builtin:git-download";
const MANTLE_BUILTIN_SYSTEM: &str = "builtin";
const NIX_DERIVATION_BUILTIN: &str = "nix.derivation";
const OUTPUT_ENV_SEPARATOR: &str = " ";
const FOREIGN_DIGEST_DOMAIN: &str = "foreign-compatible";
const MANTLE_TARGET_DIGEST_DOMAIN: &str = "mantle-target";
const SHA256_ALGORITHM: &str = "sha256";
const BLAKE3_ALGORITHM: &str = "blake3";
const FIXED_OUTPUT_CONTENT_ROLE: &str = "fixed-output-content";
const TARGET_ATERM_ROLE: &str = "target-derivation-aterm";
const TARGET_HDM_ROLE: &str = "target-hash-derivation-modulo";
const FETCH_MODE_FLAT: &str = "flat";
const FETCH_MODE_RECURSIVE: &str = "recursive";
const GIT_EXPORT_POLICY: &str = "checkout-no-dot-git";
const MAX_FOREIGN_FETCH_CANDIDATES: usize = 16;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ExactForeignPathMaps {
    pub(crate) derivations: BTreeMap<String, String>,
    pub(crate) outputs: BTreeMap<String, String>,
    pub(crate) sources: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct CompiledSourceRequirement {
    pub(crate) payload_id: String,
    pub(crate) foreign_path: String,
    pub(crate) target_path: String,
    pub(crate) descriptor_digest: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct CompiledDigestFact {
    pub(crate) domain: String,
    pub(crate) algorithm: String,
    pub(crate) role: String,
    pub(crate) value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub(crate) enum CompiledForeignBuiltin {
    NativeDerivation,
    FixedOutput,
    Download {
        candidates: Vec<String>,
        mode: String,
        executable: bool,
        content_digest: CompiledDigestFact,
    },
    GitDownload {
        candidates: Vec<String>,
        revision: String,
        export_policy: String,
        content_digest: CompiledDigestFact,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct CompiledForeignUnit {
    pub(crate) node_id: String,
    pub(crate) foreign_derivation: String,
    pub(crate) target_derivation: String,
    pub(crate) hdm: [u8; 32],
    pub(crate) aterm_digest: String,
    pub(crate) derivation: Derivation,
    pub(crate) execution_profile_id: String,
    pub(crate) execution_profile_digest_blake3: String,
    pub(crate) builtin: CompiledForeignBuiltin,
    pub(crate) digest_facts: Vec<CompiledDigestFact>,
    pub(crate) declared_references: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct CompiledForeignGraph {
    pub(crate) target_store_prefix: String,
    pub(crate) dependency_order: Vec<String>,
    pub(crate) roots: BTreeMap<String, String>,
    pub(crate) path_maps: ExactForeignPathMaps,
    pub(crate) source_requirements: Vec<CompiledSourceRequirement>,
    pub(crate) units: Vec<CompiledForeignUnit>,
}

#[derive(Clone, Debug)]
struct CompiledIdentity {
    target_derivation: String,
    outputs: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
struct BuiltinLowering {
    builder: String,
    system: String,
    arguments: Vec<String>,
    environment: BTreeMap<String, String>,
    fact: CompiledForeignBuiltin,
}

pub(crate) fn compile_foreign_graph(
    graph: &ForeignDerivationGraph,
    target_store_prefix: &str,
) -> Result<CompiledForeignGraph, ImportDiagnostic> {
    let execution_profile = foreign_profile_for_producer(&graph.producer.kind);
    compile_foreign_graph_with_profile(graph, target_store_prefix, &execution_profile)
}

pub(crate) fn compile_foreign_graph_with_profile(
    graph: &ForeignDerivationGraph,
    target_store_prefix: &str,
    execution_profile: &ExecutionProfile,
) -> Result<CompiledForeignGraph, ImportDiagnostic> {
    validate_graph(graph)?;
    validate_execution_profile(execution_profile).map_err(|error| {
        compiler_diagnostic(
            "foreign-compiler-execution-profile-invalid",
            None,
            &format!("execution profile validation failed: {error}"),
        )
    })?;
    validate_target_store_prefix(target_store_prefix)?;
    let dependency_order = foreign_dependency_order(graph)?;
    let nodes = graph.nodes.iter().map(|node| (node.node_id.as_str(), node)).collect::<BTreeMap<_, _>>();
    let payloads = graph
        .source_payloads
        .iter()
        .map(|payload| (payload.payload_id.as_str(), payload))
        .collect::<BTreeMap<_, _>>();
    let (mut path_maps, source_requirements) = compile_source_requirements(graph, target_store_prefix)?;
    let mut identities = BTreeMap::new();
    let mut known_hdms = BTreeMap::new();
    let mut units = Vec::with_capacity(dependency_order.len());

    for node_id in &dependency_order {
        let node = nodes.get(node_id.as_str()).copied().ok_or_else(|| {
            compiler_diagnostic(
                "foreign-compiler-missing-node",
                Some(node_id),
                "dependency order references an absent node",
            )
        })?;
        let mut unit = compile_node(
            node,
            graph,
            target_store_prefix,
            execution_profile,
            &payloads,
            &identities,
            &known_hdms,
            &path_maps,
        )?;
        let output_paths = exact_output_paths(&unit.derivation, target_store_prefix, &node.node_id)?;
        insert_exact_path(
            &mut path_maps.derivations,
            &node.original_derivation,
            &unit.target_derivation,
            "derivation",
            Some(&node.node_id),
        )?;
        for (output_name, foreign_output) in &node.outputs {
            let target_output = output_paths.get(output_name).ok_or_else(|| {
                compiler_diagnostic(
                    "foreign-compiler-output-name-drift",
                    Some(&node.node_id),
                    "resolved derivation omitted a declared output",
                )
            })?;
            insert_exact_path(
                &mut path_maps.outputs,
                &foreign_output.path,
                target_output,
                "output",
                Some(&node.node_id),
            )?;
        }
        validate_cross_map_collisions(&path_maps, Some(&node.node_id))?;
        known_hdms.insert(unit.target_derivation.clone(), unit.hdm);
        identities.insert(node.node_id.clone(), CompiledIdentity {
            target_derivation: unit.target_derivation.clone(),
            outputs: output_paths,
        });
        unit.declared_references = node.declared_references.clone();
        units.push(unit);
    }
    validate_compiled_coverage(&dependency_order, &units)?;

    let exact_map = combined_path_map(&path_maps, None)?;
    for unit in &mut units {
        unit.declared_references =
            rewrite_values(&unit.declared_references, &exact_map, &graph.source_store_prefixes, Some(&unit.node_id))?;
        validate_compiled_unit_references(unit, &graph.source_store_prefixes)?;
    }
    let roots = graph
        .root_derivation_ids
        .iter()
        .map(|root| {
            identities
                .get(root)
                .map(|identity| (root.clone(), identity.target_derivation.clone()))
                .ok_or_else(|| {
                    compiler_diagnostic(
                        "foreign-compiler-partial-plan",
                        Some(root),
                        "compiled graph omitted a selected root",
                    )
                })
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;

    debug_assert_eq!(dependency_order.len(), units.len());
    debug_assert_eq!(identities.len(), units.len());
    debug_assert!(known_hdms.len() >= units.len());
    Ok(CompiledForeignGraph {
        target_store_prefix: target_store_prefix.to_string(),
        dependency_order,
        roots,
        path_maps,
        source_requirements,
        units,
    })
}

pub(crate) fn foreign_dependency_order(graph: &ForeignDerivationGraph) -> Result<Vec<String>, ImportDiagnostic> {
    validate_graph_edges(graph)?;
    let nodes = graph.nodes.iter().map(|node| (node.node_id.as_str(), node)).collect::<BTreeMap<_, _>>();
    let reachable = reachable_nodes(graph, &nodes)?;
    let mut dependency_counts = reachable.iter().map(|node_id| (node_id.clone(), 0usize)).collect::<BTreeMap<_, _>>();
    let mut dependents = BTreeMap::<String, BTreeSet<String>>::new();

    for node_id in &reachable {
        let node = nodes.get(node_id.as_str()).copied().ok_or_else(|| {
            compiler_diagnostic("foreign-compiler-missing-node", Some(node_id), "reachable node is absent")
        })?;
        let dependency_nodes = node.input_derivations.iter().map(|edge| edge.node_id.clone()).collect::<BTreeSet<_>>();
        dependency_counts.insert(node_id.clone(), dependency_nodes.len());
        for dependency in dependency_nodes {
            dependents.entry(dependency).or_default().insert(node_id.clone());
        }
    }

    let mut ready = dependency_counts
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(node_id, _)| node_id.clone())
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(reachable.len());
    while let Some(node_id) = ready.iter().next().cloned() {
        let removed = ready.remove(&node_id);
        assert!(removed, "ready node must be removable");
        order.push(node_id.clone());
        if let Some(node_dependents) = dependents.get(&node_id) {
            for dependent in node_dependents {
                let count = dependency_counts.get_mut(dependent).ok_or_else(|| {
                    compiler_diagnostic(
                        "foreign-compiler-order-state-invalid",
                        Some(dependent),
                        "dependent node has no dependency count",
                    )
                })?;
                if *count == 0 {
                    return Err(compiler_diagnostic(
                        "foreign-compiler-order-state-invalid",
                        Some(dependent),
                        "dependent node count underflowed",
                    ));
                }
                *count -= 1;
                if *count == 0 {
                    ready.insert(dependent.clone());
                }
            }
        }
    }
    if order.len() != reachable.len() {
        return Err(compiler_diagnostic(
            "foreign-compiler-cycle",
            None,
            "reachable foreign derivation graph contains a cycle",
        ));
    }
    validate_order_coverage(&reachable, &order)?;
    debug_assert_eq!(order.len(), reachable.len());
    Ok(order)
}

fn validate_graph_edges(graph: &ForeignDerivationGraph) -> Result<(), ImportDiagnostic> {
    let nodes = graph.nodes.iter().map(|node| (node.node_id.as_str(), node)).collect::<BTreeMap<_, _>>();
    for node in &graph.nodes {
        let mut edges = BTreeSet::new();
        for edge in &node.input_derivations {
            if !edges.insert((edge.node_id.as_str(), edge.output_name.as_str())) {
                return Err(compiler_diagnostic(
                    "foreign-compiler-duplicate-edge",
                    Some(&node.node_id),
                    "node repeats the same dependency output edge",
                ));
            }
            let dependency = nodes.get(edge.node_id.as_str()).copied().ok_or_else(|| {
                compiler_diagnostic(
                    "foreign-compiler-missing-node",
                    Some(&node.node_id),
                    "dependency edge references an absent node",
                )
            })?;
            if !dependency.outputs.contains_key(&edge.output_name) {
                return Err(compiler_diagnostic(
                    "foreign-compiler-unknown-output",
                    Some(&node.node_id),
                    "dependency edge selects an undeclared output",
                ));
            }
        }
    }
    Ok(())
}

fn reachable_nodes(
    graph: &ForeignDerivationGraph,
    nodes: &BTreeMap<&str, &ForeignDerivationNode>,
) -> Result<BTreeSet<String>, ImportDiagnostic> {
    let mut roots = BTreeSet::new();
    for root in &graph.root_derivation_ids {
        if !roots.insert(root.clone()) {
            return Err(compiler_diagnostic(
                "foreign-compiler-duplicate-root",
                Some(root),
                "root derivation is repeated",
            ));
        }
    }
    let mut pending = roots.iter().cloned().collect::<Vec<_>>();
    let mut reachable = BTreeSet::new();
    while let Some(node_id) = pending.pop() {
        if !reachable.insert(node_id.clone()) {
            continue;
        }
        let node = nodes.get(node_id.as_str()).copied().ok_or_else(|| {
            compiler_diagnostic("foreign-compiler-missing-node", Some(&node_id), "reachable derivation is absent")
        })?;
        for edge in &node.input_derivations {
            pending.push(edge.node_id.clone());
        }
    }
    debug_assert!(!reachable.is_empty());
    Ok(reachable)
}

fn validate_order_coverage(reachable: &BTreeSet<String>, order: &[String]) -> Result<(), ImportDiagnostic> {
    let ordered = order.iter().cloned().collect::<BTreeSet<_>>();
    if ordered.len() != order.len() || ordered != *reachable {
        return Err(compiler_diagnostic(
            "foreign-compiler-partial-plan",
            None,
            "dependency order does not cover each reachable node exactly once",
        ));
    }
    Ok(())
}

fn compile_source_requirements(
    graph: &ForeignDerivationGraph,
    target_store_prefix: &str,
) -> Result<(ExactForeignPathMaps, Vec<CompiledSourceRequirement>), ImportDiagnostic> {
    let mut path_maps = ExactForeignPathMaps::default();
    let mut requirements = Vec::with_capacity(graph.source_payloads.len());
    let mut payloads = graph.source_payloads.iter().collect::<Vec<_>>();
    payloads.sort_by(|left, right| left.payload_id.cmp(&right.payload_id));
    for payload in payloads {
        let foreign_path = parse_foreign_store_object(&payload.content_ref, &graph.source_store_prefixes, None)?;
        let descriptor = serde_json::to_vec(payload).map_err(|error| {
            compiler_diagnostic(
                "foreign-compiler-source-serialization-failed",
                None,
                &format!("source payload serialization failed: {error}"),
            )
        })?;
        let target: StorePath<String> = build_text_path_with_store_dir(
            foreign_path.name().as_str(),
            &descriptor,
            Vec::<String>::new(),
            target_store_prefix,
        )
        .map_err(|error| {
            compiler_diagnostic(
                "foreign-compiler-source-path-failed",
                None,
                &format!("target source path computation failed: {error}"),
            )
        })?;
        let target_path = target.to_absolute_path_with_prefix(target_store_prefix);
        insert_exact_path(&mut path_maps.sources, &payload.content_ref, &target_path, "source", None)?;
        requirements.push(CompiledSourceRequirement {
            payload_id: payload.payload_id.clone(),
            foreign_path: payload.content_ref.clone(),
            target_path,
            descriptor_digest: blake3::hash(&descriptor).to_hex().to_string(),
        });
    }
    validate_cross_map_collisions(&path_maps, None)?;
    debug_assert_eq!(requirements.len(), graph.source_payloads.len());
    Ok((path_maps, requirements))
}

#[allow(clippy::too_many_arguments)]
fn compile_node(
    node: &ForeignDerivationNode,
    graph: &ForeignDerivationGraph,
    target_store_prefix: &str,
    execution_profile: &ExecutionProfile,
    payloads: &BTreeMap<&str, &SourcePayload>,
    identities: &BTreeMap<String, CompiledIdentity>,
    known_hdms: &BTreeMap<String, [u8; 32]>,
    path_maps: &ExactForeignPathMaps,
) -> Result<CompiledForeignUnit, ImportDiagnostic> {
    validate_node_compile_surface(node)?;
    let builtin = lower_foreign_builtin(node, payloads)?;
    let exact_map = combined_path_map(path_maps, Some(&node.node_id))?;
    let builder =
        rewrite_store_objects(&builtin.builder, &exact_map, &graph.source_store_prefixes, Some(&node.node_id))?;
    let arguments = rewrite_values(&builtin.arguments, &exact_map, &graph.source_store_prefixes, Some(&node.node_id))?;
    let system = rewrite_store_objects(&builtin.system, &exact_map, &graph.source_store_prefixes, Some(&node.node_id))?;
    let outputs = compile_outputs(node)?;
    let input_derivations = compile_input_derivations(node, identities, target_store_prefix)?;
    let input_sources = compile_input_sources(node, payloads, path_maps, target_store_prefix)?;
    let environment = compile_environment(node, graph, &exact_map, &builder, &system, &builtin.environment)?;
    let mut derivation = Derivation {
        arguments,
        builder,
        environment,
        input_derivations,
        input_sources,
        outputs,
        system,
    };
    let execution_profile_digest_blake3 =
        bind_execution_profile(&mut derivation, execution_profile).map_err(|error| {
            compiler_diagnostic(
                "foreign-compiler-execution-profile-binding-invalid",
                Some(&node.node_id),
                &format!("execution profile binding failed: {error}"),
            )
        })?;
    let expected_profile_digest = execution_profile_digest(execution_profile).map_err(|error| {
        compiler_diagnostic(
            "foreign-compiler-execution-profile-invalid",
            Some(&node.node_id),
            &format!("execution profile identity failed: {error}"),
        )
    })?;
    assert_eq!(execution_profile_digest_blake3, expected_profile_digest);
    debug_assert_eq!(
        derivation
            .environment
            .get(EXECUTION_PROFILE_BINDING_ENV)
            .map(|value| String::from_utf8_lossy(value).into_owned()),
        Some(execution_profile_digest_blake3.clone())
    );
    let registration = crunch_glue::resolve_derivation_registration(ResolvedDerivationRequest {
        name: &node.name,
        derivation,
        input_hdms: known_hdms,
        store_dir: target_store_prefix,
        addressing_mode: FOREIGN_INPUT_ADDRESSING_MODE,
        dynamic_plan_outputs: Vec::new(),
        provenance_claims: None,
    })
    .map_err(|error| {
        compiler_diagnostic(
            "foreign-compiler-registration-failed",
            Some(&node.node_id),
            &format!("resolved registration failed: {error}"),
        )
    })?;
    let target_derivation = registration.drv_path.to_absolute_path_with_prefix(target_store_prefix);
    let aterm_digest = HEXLOWER.encode(&registration.aterm_hash);
    let digest_facts = compiled_digest_facts(node.fixed_output.as_ref(), &registration.hdm, &aterm_digest)?;
    debug_assert!(registration.derivation.outputs.values().all(|output| output.path.is_some()));
    Ok(CompiledForeignUnit {
        node_id: node.node_id.clone(),
        foreign_derivation: node.original_derivation.clone(),
        target_derivation,
        hdm: registration.hdm,
        aterm_digest,
        derivation: registration.derivation,
        execution_profile_id: execution_profile.profile_id.clone(),
        execution_profile_digest_blake3,
        builtin: builtin.fact,
        digest_facts,
        declared_references: Vec::new(),
    })
}

fn validate_node_compile_surface(node: &ForeignDerivationNode) -> Result<(), ImportDiagnostic> {
    if node.unsupported_features.iter().any(|feature| feature.mandatory) {
        return Err(compiler_diagnostic(
            "foreign-compiler-unsupported-feature",
            Some(&node.node_id),
            "node declares a mandatory unsupported feature",
        ));
    }
    match (node.builtin.as_str(), node.fixed_output.is_some()) {
        (NIX_DERIVATION_BUILTIN, false)
        | (FIXED_OUTPUT_FETCH_BUILTIN, true)
        | (FOREIGN_DOWNLOAD_BUILTIN, true)
        | (FOREIGN_GIT_DOWNLOAD_BUILTIN, true) => Ok(()),
        (
            NIX_DERIVATION_BUILTIN
            | FIXED_OUTPUT_FETCH_BUILTIN
            | FOREIGN_DOWNLOAD_BUILTIN
            | FOREIGN_GIT_DOWNLOAD_BUILTIN,
            _,
        ) => Err(compiler_diagnostic(
            "foreign-compiler-builtin-shape-mismatch",
            Some(&node.node_id),
            "foreign builtin does not match fixed-output metadata",
        )),
        _ => Err(compiler_diagnostic(
            "foreign-compiler-unsupported-builtin",
            Some(&node.node_id),
            "foreign builtin has no native compiler mapping",
        )),
    }
}

fn lower_foreign_builtin(
    node: &ForeignDerivationNode,
    payloads: &BTreeMap<&str, &SourcePayload>,
) -> Result<BuiltinLowering, ImportDiagnostic> {
    match node.builtin.as_str() {
        NIX_DERIVATION_BUILTIN => Ok(native_builtin_lowering(node, CompiledForeignBuiltin::NativeDerivation)),
        FIXED_OUTPUT_FETCH_BUILTIN => Ok(BuiltinLowering {
            builder: FETCH_BUILDER.to_string(),
            system: MANTLE_BUILTIN_SYSTEM.to_string(),
            arguments: Vec::new(),
            environment: BTreeMap::new(),
            fact: CompiledForeignBuiltin::FixedOutput,
        }),
        FOREIGN_DOWNLOAD_BUILTIN => lower_download_builtin(node, payloads),
        FOREIGN_GIT_DOWNLOAD_BUILTIN => lower_git_download_builtin(node, payloads),
        _ => Err(compiler_diagnostic(
            "foreign-compiler-unsupported-builtin",
            Some(&node.node_id),
            "foreign builtin has no native compiler mapping",
        )),
    }
}

fn native_builtin_lowering(node: &ForeignDerivationNode, fact: CompiledForeignBuiltin) -> BuiltinLowering {
    BuiltinLowering {
        builder: node.builder.clone(),
        system: node.system.clone(),
        arguments: node.args.clone(),
        environment: BTreeMap::new(),
        fact,
    }
}

fn lower_download_builtin(
    node: &ForeignDerivationNode,
    payloads: &BTreeMap<&str, &SourcePayload>,
) -> Result<BuiltinLowering, ImportDiagnostic> {
    let fixed_output = required_sha256_fixed_output(node)?;
    let mode = required_fetch_mode(node, fixed_output)?;
    let executable = required_boolean_environment(node, "executable")?;
    if executable && fixed_output.recursive {
        return Err(compiler_diagnostic(
            "foreign-compiler-executable-mode-mismatch",
            Some(&node.node_id),
            "executable downloads require flat fixed-output mode",
        ));
    }
    let candidates = ordered_fetch_candidates(node, payloads)?;
    let content_digest = fixed_output_digest_fact(fixed_output);
    let mut environment = BTreeMap::from([
        ("url".to_string(), candidates[0].clone()),
        ("mode".to_string(), mode.clone()),
        (FOREIGN_FETCH_CANDIDATES_ENV.to_string(), serialize_candidates(&candidates, &node.node_id)?),
    ]);
    if fixed_output.recursive {
        environment.insert("unpack".to_string(), "1".to_string());
    }
    if executable {
        environment.insert("executable".to_string(), "1".to_string());
    }
    Ok(BuiltinLowering {
        builder: FETCH_BUILDER.to_string(),
        system: MANTLE_BUILTIN_SYSTEM.to_string(),
        arguments: Vec::new(),
        environment,
        fact: CompiledForeignBuiltin::Download {
            candidates,
            mode,
            executable,
            content_digest,
        },
    })
}

fn lower_git_download_builtin(
    node: &ForeignDerivationNode,
    payloads: &BTreeMap<&str, &SourcePayload>,
) -> Result<BuiltinLowering, ImportDiagnostic> {
    let fixed_output = required_sha256_fixed_output(node)?;
    let mode = required_fetch_mode(node, fixed_output)?;
    if mode != FETCH_MODE_RECURSIVE {
        return Err(compiler_diagnostic(
            "foreign-compiler-git-mode-mismatch",
            Some(&node.node_id),
            "Git downloads require recursive fixed-output mode",
        ));
    }
    let revision = required_environment(node, "rev")?;
    let export_policy = required_environment(node, "exportPolicy")?;
    if export_policy != GIT_EXPORT_POLICY {
        return Err(compiler_diagnostic(
            "foreign-compiler-git-export-policy-unsupported",
            Some(&node.node_id),
            "Git download export policy is unsupported",
        ));
    }
    let candidates = ordered_fetch_candidates(node, payloads)?;
    let content_digest = fixed_output_digest_fact(fixed_output);
    let environment = BTreeMap::from([
        ("type".to_string(), "git".to_string()),
        ("url".to_string(), candidates[0].clone()),
        ("rev".to_string(), revision.clone()),
        ("exportPolicy".to_string(), export_policy.clone()),
        (FOREIGN_FETCH_CANDIDATES_ENV.to_string(), serialize_candidates(&candidates, &node.node_id)?),
    ]);
    Ok(BuiltinLowering {
        builder: FETCH_BUILDER.to_string(),
        system: MANTLE_BUILTIN_SYSTEM.to_string(),
        arguments: Vec::new(),
        environment,
        fact: CompiledForeignBuiltin::GitDownload {
            candidates,
            revision,
            export_policy,
            content_digest,
        },
    })
}

fn required_sha256_fixed_output(node: &ForeignDerivationNode) -> Result<&FixedOutputMetadata, ImportDiagnostic> {
    let fixed_output = node.fixed_output.as_ref().ok_or_else(|| {
        compiler_diagnostic(
            "foreign-compiler-builtin-shape-mismatch",
            Some(&node.node_id),
            "foreign download builtin requires fixed-output metadata",
        )
    })?;
    if fixed_output.algorithm != SHA256_ALGORITHM {
        return Err(compiler_diagnostic(
            "foreign-compiler-digest-domain-mismatch",
            Some(&node.node_id),
            "foreign download content identity must use declared SHA-256",
        ));
    }
    Ok(fixed_output)
}

fn required_fetch_mode(
    node: &ForeignDerivationNode,
    fixed_output: &FixedOutputMetadata,
) -> Result<String, ImportDiagnostic> {
    let declared = required_environment(node, "mode")?;
    let expected = if fixed_output.recursive {
        FETCH_MODE_RECURSIVE
    } else {
        FETCH_MODE_FLAT
    };
    if declared != expected {
        return Err(compiler_diagnostic(
            "foreign-compiler-fetch-mode-mismatch",
            Some(&node.node_id),
            "foreign fetch mode differs from fixed-output metadata",
        ));
    }
    Ok(declared)
}

fn required_boolean_environment(node: &ForeignDerivationNode, field: &str) -> Result<bool, ImportDiagnostic> {
    match required_environment(node, field)?.as_str() {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(compiler_diagnostic(
            "foreign-compiler-boolean-field-invalid",
            Some(&node.node_id),
            &format!("foreign builtin field {field} must be 0 or 1"),
        )),
    }
}

fn required_environment(node: &ForeignDerivationNode, field: &str) -> Result<String, ImportDiagnostic> {
    node.env.get(field).filter(|value| !value.trim().is_empty()).cloned().ok_or_else(|| {
        compiler_diagnostic(
            "foreign-compiler-builtin-field-missing",
            Some(&node.node_id),
            &format!("foreign builtin field {field} is required"),
        )
    })
}

fn ordered_fetch_candidates(
    node: &ForeignDerivationNode,
    payloads: &BTreeMap<&str, &SourcePayload>,
) -> Result<Vec<String>, ImportDiagnostic> {
    let mut candidates = Vec::new();
    if let Some(primary) = node.env.get("url").filter(|value| !value.trim().is_empty()) {
        candidates.push(primary.clone());
    }
    for source_ref in &node.source_refs {
        let payload = payloads.get(source_ref.payload_id.as_str()).copied().ok_or_else(|| {
            compiler_diagnostic(
                "foreign-compiler-missing-source",
                Some(&node.node_id),
                "fetch candidate source payload is absent",
            )
        })?;
        candidates.extend(payload.mirrors.iter().cloned());
    }
    if candidates.is_empty() || candidates.len() > MAX_FOREIGN_FETCH_CANDIDATES {
        return Err(compiler_diagnostic(
            "foreign-compiler-fetch-candidate-count-invalid",
            Some(&node.node_id),
            "foreign fetch candidate count is outside supported limits",
        ));
    }
    let mut seen = BTreeSet::new();
    for candidate in &candidates {
        if candidate.trim().is_empty() || url::Url::parse(candidate).is_err() {
            return Err(compiler_diagnostic(
                "foreign-compiler-fetch-candidate-invalid",
                Some(&node.node_id),
                "foreign fetch candidate is not a valid URL",
            ));
        }
        if !seen.insert(candidate) {
            return Err(compiler_diagnostic(
                "foreign-compiler-fetch-candidate-duplicate",
                Some(&node.node_id),
                "foreign fetch candidates contain a duplicate URL",
            ));
        }
    }
    debug_assert!(!candidates.is_empty());
    debug_assert!(candidates.len() <= MAX_FOREIGN_FETCH_CANDIDATES);
    Ok(candidates)
}

fn serialize_candidates(candidates: &[String], node_id: &str) -> Result<String, ImportDiagnostic> {
    serde_json::to_string(candidates).map_err(|error| {
        compiler_diagnostic(
            "foreign-compiler-fetch-candidates-serialization-failed",
            Some(node_id),
            &format!("foreign fetch candidates serialization failed: {error}"),
        )
    })
}

fn fixed_output_digest_fact(fixed_output: &FixedOutputMetadata) -> CompiledDigestFact {
    CompiledDigestFact {
        domain: FOREIGN_DIGEST_DOMAIN.to_string(),
        algorithm: fixed_output.algorithm.clone(),
        role: FIXED_OUTPUT_CONTENT_ROLE.to_string(),
        value: fixed_output.digest.clone(),
    }
}

fn compiled_digest_facts(
    fixed_output: Option<&FixedOutputMetadata>,
    hdm: &[u8; 32],
    aterm_digest: &str,
) -> Result<Vec<CompiledDigestFact>, ImportDiagnostic> {
    let mut facts = Vec::new();
    if let Some(fixed_output) = fixed_output {
        facts.push(fixed_output_digest_fact(fixed_output));
    }
    facts.push(CompiledDigestFact {
        domain: MANTLE_TARGET_DIGEST_DOMAIN.to_string(),
        algorithm: BLAKE3_ALGORITHM.to_string(),
        role: TARGET_HDM_ROLE.to_string(),
        value: HEXLOWER.encode(hdm),
    });
    facts.push(CompiledDigestFact {
        domain: MANTLE_TARGET_DIGEST_DOMAIN.to_string(),
        algorithm: BLAKE3_ALGORITHM.to_string(),
        role: TARGET_ATERM_ROLE.to_string(),
        value: aterm_digest.to_string(),
    });
    validate_digest_facts(&facts)?;
    Ok(facts)
}

pub(crate) fn validate_digest_facts(facts: &[CompiledDigestFact]) -> Result<(), ImportDiagnostic> {
    for fact in facts {
        let valid = match fact.role.as_str() {
            FIXED_OUTPUT_CONTENT_ROLE => fact.domain == FOREIGN_DIGEST_DOMAIN && fact.algorithm != BLAKE3_ALGORITHM,
            TARGET_HDM_ROLE | TARGET_ATERM_ROLE => {
                fact.domain == MANTLE_TARGET_DIGEST_DOMAIN && fact.algorithm == BLAKE3_ALGORITHM
            }
            _ => false,
        };
        if !valid {
            return Err(compiler_diagnostic(
                "foreign-compiler-digest-domain-mismatch",
                None,
                "digest domain, algorithm, and role do not match",
            ));
        }
    }
    Ok(())
}

fn compile_outputs(node: &ForeignDerivationNode) -> Result<BTreeMap<String, Output>, ImportDiagnostic> {
    validate_fixed_output_declaration(node)?;
    let ca_hash = node.fixed_output.as_ref().map(parse_fixed_output).transpose()?;
    let outputs = node
        .outputs
        .keys()
        .map(|output_name| {
            let output_ca_hash = if output_name == "out" { ca_hash.clone() } else { None };
            (output_name.clone(), Output {
                path: None,
                ca_hash: output_ca_hash,
            })
        })
        .collect::<BTreeMap<_, _>>();
    debug_assert_eq!(outputs.len(), node.outputs.len());
    Ok(outputs)
}

fn validate_fixed_output_declaration(node: &ForeignDerivationNode) -> Result<(), ImportDiagnostic> {
    let Some(fixed_output) = &node.fixed_output else {
        if node.outputs.values().any(|output| output.hash.is_some()) {
            return Err(compiler_diagnostic(
                "foreign-compiler-output-hash-drift",
                Some(&node.node_id),
                "non-fixed derivation declares an output hash",
            ));
        }
        return Ok(());
    };
    if node.outputs.len() != 1 || !node.outputs.contains_key("out") {
        return Err(compiler_diagnostic(
            "foreign-compiler-output-name-drift",
            Some(&node.node_id),
            "fixed-output derivation must declare exactly the out output",
        ));
    }
    let declared_hash = node.outputs["out"].hash.as_deref().ok_or_else(|| {
        compiler_diagnostic(
            "foreign-compiler-output-hash-drift",
            Some(&node.node_id),
            "fixed-output declaration omits its output hash",
        )
    })?;
    if declared_hash != fixed_output.digest {
        return Err(compiler_diagnostic(
            "foreign-compiler-output-hash-drift",
            Some(&node.node_id),
            "fixed-output metadata differs from the declared output hash",
        ));
    }
    Ok(())
}

fn parse_fixed_output(metadata: &FixedOutputMetadata) -> Result<CAHash, ImportDiagnostic> {
    let algorithm: HashAlgo = metadata.algorithm.parse().map_err(|_| {
        compiler_diagnostic(
            "foreign-compiler-fixed-output-hash-invalid",
            None,
            "fixed-output hash algorithm is unsupported",
        )
    })?;
    let digest = if metadata.digest.contains('-') {
        NixHash::from_sri(&metadata.digest)
    } else {
        let bytes = HEXLOWER.decode(metadata.digest.as_bytes()).map_err(|error| {
            compiler_diagnostic(
                "foreign-compiler-fixed-output-hash-invalid",
                None,
                &format!("fixed-output digest is not lowercase hexadecimal: {error}"),
            )
        })?;
        NixHash::from_algo_and_digest(algorithm, &bytes)
    }
    .map_err(|error| {
        compiler_diagnostic(
            "foreign-compiler-fixed-output-hash-invalid",
            None,
            &format!("fixed-output digest is invalid: {error}"),
        )
    })?;
    if digest.algo() != algorithm {
        return Err(compiler_diagnostic(
            "foreign-compiler-fixed-output-hash-invalid",
            None,
            "fixed-output digest algorithm differs from metadata",
        ));
    }
    if metadata.recursive {
        Ok(CAHash::Nar(digest))
    } else {
        Ok(CAHash::Flat(digest))
    }
}

fn compile_input_derivations(
    node: &ForeignDerivationNode,
    identities: &BTreeMap<String, CompiledIdentity>,
    target_store_prefix: &str,
) -> Result<BTreeMap<StorePath<String>, BTreeSet<String>>, ImportDiagnostic> {
    let mut input_derivations = BTreeMap::<StorePath<String>, BTreeSet<String>>::new();
    for edge in &node.input_derivations {
        let identity = identities.get(&edge.node_id).ok_or_else(|| {
            compiler_diagnostic(
                "foreign-compiler-dependency-not-ready",
                Some(&node.node_id),
                "dependency identity is unavailable before parent compilation",
            )
        })?;
        if !identity.outputs.contains_key(&edge.output_name) {
            return Err(compiler_diagnostic(
                "foreign-compiler-output-name-drift",
                Some(&node.node_id),
                "compiled dependency omitted the selected output",
            ));
        }
        let path = parse_target_store_object(&identity.target_derivation, target_store_prefix, Some(&node.node_id))?;
        input_derivations.entry(path).or_default().insert(edge.output_name.clone());
    }
    Ok(input_derivations)
}

fn compile_input_sources(
    node: &ForeignDerivationNode,
    payloads: &BTreeMap<&str, &SourcePayload>,
    path_maps: &ExactForeignPathMaps,
    target_store_prefix: &str,
) -> Result<BTreeSet<StorePath<String>>, ImportDiagnostic> {
    let mut seen = BTreeSet::new();
    let mut input_sources = BTreeSet::new();
    for source_ref in &node.source_refs {
        validate_unique_source_ref(node, source_ref, &mut seen)?;
        let payload = payloads.get(source_ref.payload_id.as_str()).copied().ok_or_else(|| {
            compiler_diagnostic(
                "foreign-compiler-missing-source",
                Some(&node.node_id),
                "source reference points at an absent payload",
            )
        })?;
        let target = path_maps.sources.get(&payload.content_ref).ok_or_else(|| {
            compiler_diagnostic(
                "foreign-compiler-missing-source-map",
                Some(&node.node_id),
                "source payload has no exact target path",
            )
        })?;
        input_sources.insert(parse_target_store_object(target, target_store_prefix, Some(&node.node_id))?);
    }
    Ok(input_sources)
}

fn validate_unique_source_ref<'a>(
    node: &ForeignDerivationNode,
    source_ref: &'a SourceRef,
    seen: &mut BTreeSet<(&'a str, &'a str)>,
) -> Result<(), ImportDiagnostic> {
    if !seen.insert((source_ref.payload_id.as_str(), source_ref.field.as_str())) {
        return Err(compiler_diagnostic(
            "foreign-compiler-duplicate-source-ref",
            Some(&node.node_id),
            "node repeats the same source reference",
        ));
    }
    Ok(())
}

fn compile_environment(
    node: &ForeignDerivationNode,
    graph: &ForeignDerivationGraph,
    exact_map: &BTreeMap<String, String>,
    builder: &str,
    system: &str,
    builtin_environment: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, BString>, ImportDiagnostic> {
    let mut environment = BTreeMap::new();
    for (key, value) in &node.env {
        if node.outputs.contains_key(key) {
            environment.insert(key.clone(), BString::from(""));
            continue;
        }
        if matches!(key.as_str(), "builder" | "name" | "outputs" | "system") {
            continue;
        }
        let rewritten = rewrite_store_objects(value, exact_map, &graph.source_store_prefixes, Some(&node.node_id))?;
        environment.insert(key.clone(), rewritten.into_bytes().into());
    }
    for (key, value) in builtin_environment {
        let rewritten = rewrite_store_objects(value, exact_map, &graph.source_store_prefixes, Some(&node.node_id))?;
        environment.insert(key.clone(), rewritten.into_bytes().into());
    }
    environment.insert("system".to_string(), system.as_bytes().into());
    environment.insert("builder".to_string(), builder.as_bytes().into());
    environment.insert("name".to_string(), node.name.as_bytes().into());
    for output_name in node.outputs.keys() {
        environment.insert(output_name.clone(), BString::from(""));
    }
    environment.insert(
        "outputs".to_string(),
        node.outputs.keys().cloned().collect::<Vec<_>>().join(OUTPUT_ENV_SEPARATOR).into_bytes().into(),
    );
    Ok(environment)
}

fn exact_output_paths(
    derivation: &Derivation,
    target_store_prefix: &str,
    node_id: &str,
) -> Result<BTreeMap<String, String>, ImportDiagnostic> {
    derivation
        .outputs
        .iter()
        .map(|(output_name, output)| {
            output
                .path
                .as_ref()
                .map(|path| (output_name.clone(), path.to_absolute_path_with_prefix(target_store_prefix)))
                .ok_or_else(|| {
                    compiler_diagnostic(
                        "foreign-compiler-output-path-unresolved",
                        Some(node_id),
                        "resolved registration did not produce an exact output path",
                    )
                })
        })
        .collect()
}

fn validate_compiled_coverage(
    dependency_order: &[String],
    units: &[CompiledForeignUnit],
) -> Result<(), ImportDiagnostic> {
    let expected = dependency_order.iter().cloned().collect::<BTreeSet<_>>();
    let actual = units.iter().map(|unit| unit.node_id.clone()).collect::<BTreeSet<_>>();
    if expected.len() != dependency_order.len() || actual.len() != units.len() || expected != actual {
        return Err(compiler_diagnostic(
            "foreign-compiler-partial-plan",
            None,
            "compiled units do not cover dependency order exactly once",
        ));
    }
    Ok(())
}

fn insert_exact_path(
    map: &mut BTreeMap<String, String>,
    foreign_path: &str,
    target_path: &str,
    object_kind: &str,
    node_id: Option<&str>,
) -> Result<(), ImportDiagnostic> {
    if map.insert(foreign_path.to_string(), target_path.to_string()).is_some() {
        return Err(compiler_diagnostic(
            "foreign-compiler-duplicate-map-entry",
            node_id,
            &format!("foreign {object_kind} path has more than one mapping entry"),
        ));
    }
    Ok(())
}

fn validate_cross_map_collisions(
    path_maps: &ExactForeignPathMaps,
    node_id: Option<&str>,
) -> Result<(), ImportDiagnostic> {
    let expected = path_maps
        .derivations
        .len()
        .checked_add(path_maps.outputs.len())
        .and_then(|count| count.checked_add(path_maps.sources.len()))
        .ok_or_else(|| {
            compiler_diagnostic("foreign-compiler-map-count-overflow", node_id, "foreign path map count overflowed")
        })?;
    let combined = combined_path_map(path_maps, node_id)?;
    if combined.len() != expected {
        return Err(compiler_diagnostic(
            "foreign-compiler-map-collision",
            node_id,
            "one foreign path appears in more than one object map",
        ));
    }
    Ok(())
}

fn combined_path_map(
    path_maps: &ExactForeignPathMaps,
    node_id: Option<&str>,
) -> Result<BTreeMap<String, String>, ImportDiagnostic> {
    let mut combined = BTreeMap::new();
    for map in [&path_maps.derivations, &path_maps.outputs, &path_maps.sources] {
        for (foreign_path, target_path) in map {
            if combined.insert(foreign_path.clone(), target_path.clone()).is_some() {
                return Err(compiler_diagnostic(
                    "foreign-compiler-map-collision",
                    node_id,
                    "one foreign path appears in more than one object map",
                ));
            }
        }
    }
    Ok(combined)
}

fn rewrite_values(
    values: &[String],
    exact_map: &BTreeMap<String, String>,
    source_prefixes: &[String],
    node_id: Option<&str>,
) -> Result<Vec<String>, ImportDiagnostic> {
    values
        .iter()
        .map(|value| rewrite_store_objects(value, exact_map, source_prefixes, node_id))
        .collect()
}

fn rewrite_store_objects(
    value: &str,
    exact_map: &BTreeMap<String, String>,
    source_prefixes: &[String],
    node_id: Option<&str>,
) -> Result<String, ImportDiagnostic> {
    let mut rewritten = String::with_capacity(value.len());
    let mut cursor = 0usize;
    while let Some((start, prefix)) = next_store_object(value, cursor, source_prefixes) {
        rewritten.push_str(&value[cursor..start]);
        let object_start =
            start.checked_add(prefix.len()).and_then(|position| position.checked_add(1)).ok_or_else(|| {
                compiler_diagnostic(
                    "foreign-compiler-path-position-overflow",
                    node_id,
                    "foreign store path position overflowed",
                )
            })?;
        let object_end = value[object_start..]
            .char_indices()
            .take_while(|(_, character)| is_store_name_character(*character))
            .last()
            .map(|(position, character)| object_start + position + character.len_utf8())
            .unwrap_or(object_start);
        if object_end == object_start {
            return Err(compiler_diagnostic(
                "foreign-compiler-malformed-reference",
                node_id,
                "foreign store prefix is not followed by an object name",
            ));
        }
        let foreign_path = &value[start..object_end];
        let target_path = exact_map.get(foreign_path).ok_or_else(|| {
            compiler_diagnostic(
                "foreign-compiler-unknown-reference",
                node_id,
                &format!("foreign store object has no exact mapping: {foreign_path}"),
            )
        })?;
        rewritten.push_str(target_path);
        cursor = object_end;
    }
    rewritten.push_str(&value[cursor..]);
    if contains_foreign_store_object(&rewritten, source_prefixes) {
        return Err(compiler_diagnostic(
            "foreign-compiler-leftover-reference",
            node_id,
            "rewritten field retains a foreign store object",
        ));
    }
    Ok(rewritten)
}

fn next_store_object<'a>(value: &str, cursor: usize, source_prefixes: &'a [String]) -> Option<(usize, &'a str)> {
    source_prefixes
        .iter()
        .filter_map(|prefix| {
            let marker = format!("{prefix}/");
            value[cursor..].find(&marker).map(|relative| (cursor + relative, prefix.as_str()))
        })
        .min_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.len().cmp(&left.1.len())))
}

fn contains_foreign_store_object(value: &str, source_prefixes: &[String]) -> bool {
    source_prefixes.iter().any(|prefix| value.contains(&format!("{prefix}/")))
}

fn is_store_name_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.' | '_' | '?' | '=')
}

fn parse_foreign_store_object(
    path: &str,
    source_prefixes: &[String],
    node_id: Option<&str>,
) -> Result<StorePath<String>, ImportDiagnostic> {
    for prefix in source_prefixes {
        if path.starts_with(&format!("{prefix}/")) {
            return StorePath::from_absolute_path_with_prefix(path.as_bytes(), prefix).map_err(|_| {
                compiler_diagnostic(
                    "foreign-compiler-invalid-foreign-path",
                    node_id,
                    "foreign source path is not a valid store object",
                )
            });
        }
    }
    Err(compiler_diagnostic(
        "foreign-compiler-unknown-source-prefix",
        node_id,
        "foreign source path is outside declared store prefixes",
    ))
}

fn parse_target_store_object(
    path: &str,
    target_store_prefix: &str,
    node_id: Option<&str>,
) -> Result<StorePath<String>, ImportDiagnostic> {
    StorePath::from_absolute_path_with_prefix(path.as_bytes(), target_store_prefix).map_err(|_| {
        compiler_diagnostic(
            "foreign-compiler-invalid-target-path",
            node_id,
            "computed target path is not a valid store object",
        )
    })
}

fn validate_target_store_prefix(target_store_prefix: &str) -> Result<(), ImportDiagnostic> {
    if !target_store_prefix.starts_with('/') || target_store_prefix.ends_with('/') || target_store_prefix.contains("//")
    {
        return Err(compiler_diagnostic(
            "foreign-compiler-invalid-target-prefix",
            None,
            "target store prefix must be an absolute normalized path without a trailing slash",
        ));
    }
    Ok(())
}

fn validate_compiled_unit_references(
    unit: &CompiledForeignUnit,
    source_prefixes: &[String],
) -> Result<(), ImportDiagnostic> {
    let mut values = Vec::new();
    values.push(unit.target_derivation.as_str());
    values.push(unit.derivation.builder.as_str());
    values.push(unit.derivation.system.as_str());
    values.extend(unit.derivation.arguments.iter().map(String::as_str));
    values.extend(unit.declared_references.iter().map(String::as_str));
    for value in unit.derivation.environment.values() {
        let text = std::str::from_utf8(value).map_err(|_| {
            compiler_diagnostic(
                "foreign-compiler-non-utf8-environment",
                Some(&unit.node_id),
                "compiled derivation environment is not UTF-8",
            )
        })?;
        values.push(text);
    }
    if values.iter().any(|value| contains_foreign_store_object(value, source_prefixes)) {
        return Err(compiler_diagnostic(
            "foreign-compiler-leftover-reference",
            Some(&unit.node_id),
            "compiled unit retains a foreign store reference",
        ));
    }
    Ok(())
}

fn compiler_diagnostic(class: &str, node_id: Option<&str>, message: &str) -> ImportDiagnostic {
    ImportDiagnostic {
        class: class.to_string(),
        node_id: node_id.map(str::to_string),
        message: message.to_string(),
    }
}

// r[verify foreign_derivation_import.exact_graph_compilation]
// r[verify foreign_derivation_import.foreign_builtin_lowering]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::foreign_derivation_import::CacheHint;
    use crate::foreign_derivation_import::FrontendMetadata;
    use crate::foreign_derivation_import::HashDomainRecord;
    use crate::foreign_derivation_import::InputDerivationEdge;
    use crate::foreign_derivation_import::OutputDeclaration;
    use crate::foreign_derivation_import::ProducerSummary;
    use crate::foreign_derivation_import::UnsupportedFeature;

    const SOURCE_PREFIX: &str = "/nix/store";
    const TARGET_PREFIX: &str = "/mantle/store";
    const OTHER_TARGET_PREFIX: &str = "/alt/store";
    const EMPTY_HASH: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    #[test]
    fn dependency_compiler_is_exact_deterministic_and_prefix_sensitive() {
        let graph = diamond_graph();
        let first = compile_foreign_graph(&graph, TARGET_PREFIX).expect("diamond graph must compile");
        let second = compile_foreign_graph(&graph, TARGET_PREFIX).expect("repeat compilation must succeed");
        let other_prefix =
            compile_foreign_graph(&graph, OTHER_TARGET_PREFIX).expect("other target prefix must compile");

        assert_eq!(first.dependency_order, vec!["leaf", "left", "right", "root"]);
        assert_eq!(first.path_maps, second.path_maps);
        assert_eq!(first.roots, second.roots);
        assert_ne!(first.path_maps, other_prefix.path_maps);
        assert_eq!(first.units.len(), first.dependency_order.len());
        assert!(first.units.iter().all(|unit| unit.target_derivation.starts_with(TARGET_PREFIX)));

        let root = first.units.iter().find(|unit| unit.node_id == "root").expect("root unit");
        let left_target = first.path_maps.outputs[&foreign_output("left", "out")].clone();
        let left_dev_target = first.path_maps.outputs[&foreign_output("left", "dev")].clone();
        assert_eq!(root.derivation.environment["leftPath"], BString::from(format!("{left_target}/bin/tool")));
        assert_eq!(root.derivation.environment["leftDevPath"], BString::from(format!("{left_dev_target}/include")));
        assert!(
            root.derivation
                .input_derivations
                .values()
                .any(|outputs| { outputs == &BTreeSet::from(["dev".to_string(), "out".to_string()]) })
        );
        assert!(
            root.derivation
                .input_derivations
                .keys()
                .all(|path| { path.to_absolute_path_with_prefix(TARGET_PREFIX).starts_with(TARGET_PREFIX) })
        );
        assert_eq!(
            root.derivation.to_aterm_bytes_with_store_dir(TARGET_PREFIX),
            second
                .units
                .iter()
                .find(|unit| unit.node_id == "root")
                .expect("second root unit")
                .derivation
                .to_aterm_bytes_with_store_dir(TARGET_PREFIX)
        );
    }

    #[test]
    fn compiler_maps_source_descriptors_and_rewrites_suffixes() {
        let mut graph = diamond_graph();
        graph.source_payloads.push(SourcePayload {
            payload_id: "script-source".to_string(),
            kind: "store-path".to_string(),
            content_ref: foreign_source("script"),
            embedded_text: Some("echo source".to_string()),
            mirrors: Vec::new(),
        });
        let root = graph.nodes.iter_mut().find(|node| node.node_id == "root").expect("root node");
        root.source_refs.push(SourceRef {
            payload_id: "script-source".to_string(),
            field: "source-ref".to_string(),
        });
        root.args.push(format!("{}/bin/run", foreign_source("script")));

        let compiled = compile_foreign_graph(&graph, TARGET_PREFIX).expect("source graph must compile");
        let target_source = &compiled.path_maps.sources[&foreign_source("script")];
        let root_unit = compiled.units.iter().find(|unit| unit.node_id == "root").expect("root unit");
        assert!(root_unit.derivation.arguments.contains(&format!("{target_source}/bin/run")));
        assert_eq!(root_unit.derivation.input_sources.len(), 1);
        assert_eq!(compiled.source_requirements.len(), 1);
        assert_eq!(compiled.source_requirements[0].target_path, *target_source);
    }

    #[test]
    fn compiler_lowers_ordered_executable_download_to_mantle_fetch_facts() {
        let mut graph = diamond_graph();
        let primary = "https://primary.example/source.bin";
        let mirrors = vec![
            "https://mirror-a.example/source.bin".to_string(),
            "https://mirror-b.example/source.bin".to_string(),
        ];
        add_fetch_payload(&mut graph, "download-source", "download", mirrors.clone());
        let leaf = graph.nodes.iter_mut().find(|node| node.node_id == "leaf").expect("leaf");
        leaf.builtin = FOREIGN_DOWNLOAD_BUILTIN.to_string();
        leaf.fixed_output.as_mut().expect("fixed output").recursive = false;
        leaf.env.insert("url".to_string(), primary.to_string());
        leaf.env.insert("mode".to_string(), FETCH_MODE_FLAT.to_string());
        leaf.env.insert("executable".to_string(), "1".to_string());
        leaf.source_refs.push(SourceRef {
            payload_id: "download-source".to_string(),
            field: "source-ref".to_string(),
        });

        let compiled = compile_foreign_graph(&graph, TARGET_PREFIX).expect("download graph must compile");
        let leaf_unit = compiled.units.iter().find(|unit| unit.node_id == "leaf").expect("leaf unit");
        let mut expected_candidates = vec![primary.to_string()];
        expected_candidates.extend(mirrors);
        match &leaf_unit.builtin {
            CompiledForeignBuiltin::Download {
                candidates,
                mode,
                executable,
                content_digest,
            } => {
                assert_eq!(candidates, &expected_candidates);
                assert_eq!(mode, FETCH_MODE_FLAT);
                assert!(*executable);
                assert_eq!(content_digest.domain, FOREIGN_DIGEST_DOMAIN);
                assert_eq!(content_digest.algorithm, SHA256_ALGORITHM);
            }
            other => panic!("unexpected lowered builtin: {other:?}"),
        }
        assert_eq!(leaf_unit.derivation.builder, FETCH_BUILDER);
        assert_eq!(leaf_unit.derivation.system, MANTLE_BUILTIN_SYSTEM);
        assert!(matches!(
            crunch_build::fetcher::parse_fetch(&leaf_unit.derivation).expect("native fetch facts must parse"),
            crunch_build::Fetch::Executable { .. }
        ));
        assert!(leaf_unit.digest_facts.iter().any(|fact| {
            fact.role == FIXED_OUTPUT_CONTENT_ROLE
                && fact.domain == FOREIGN_DIGEST_DOMAIN
                && fact.algorithm == SHA256_ALGORITHM
        }));
        assert!(leaf_unit.digest_facts.iter().any(|fact| {
            fact.role == TARGET_ATERM_ROLE
                && fact.domain == MANTLE_TARGET_DIGEST_DOMAIN
                && fact.algorithm == BLAKE3_ALGORITHM
        }));
    }

    #[test]
    fn compiler_lowers_git_revision_and_export_policy_to_mantle_fetch_facts() {
        let mut graph = diamond_graph();
        let primary = "https://git.example/project.git";
        let revision = "0123456789abcdef0123456789abcdef01234567";
        let mirror = "https://mirror.example/project.git";
        add_fetch_payload(&mut graph, "git-source", "git", vec![mirror.to_string()]);
        let leaf = graph.nodes.iter_mut().find(|node| node.node_id == "leaf").expect("leaf");
        leaf.builtin = FOREIGN_GIT_DOWNLOAD_BUILTIN.to_string();
        leaf.env.insert("url".to_string(), primary.to_string());
        leaf.env.insert("mode".to_string(), FETCH_MODE_RECURSIVE.to_string());
        leaf.env.insert("rev".to_string(), revision.to_string());
        leaf.env.insert("exportPolicy".to_string(), GIT_EXPORT_POLICY.to_string());
        leaf.source_refs.push(SourceRef {
            payload_id: "git-source".to_string(),
            field: "source-ref".to_string(),
        });

        let compiled = compile_foreign_graph(&graph, TARGET_PREFIX).expect("Git graph must compile");
        let leaf_unit = compiled.units.iter().find(|unit| unit.node_id == "leaf").expect("leaf unit");
        match &leaf_unit.builtin {
            CompiledForeignBuiltin::GitDownload {
                candidates,
                revision: compiled_revision,
                export_policy,
                content_digest,
            } => {
                assert_eq!(candidates, &[primary.to_string(), mirror.to_string()]);
                assert_eq!(compiled_revision, revision);
                assert_eq!(export_policy, GIT_EXPORT_POLICY);
                assert_eq!(content_digest.algorithm, SHA256_ALGORITHM);
            }
            other => panic!("unexpected lowered builtin: {other:?}"),
        }
        match crunch_build::fetcher::parse_fetch(&leaf_unit.derivation).expect("native Git fetch facts must parse") {
            crunch_build::Fetch::Git { url, rev, exp_hash } => {
                assert_eq!(url, primary);
                assert_eq!(rev, revision);
                assert!(exp_hash.is_some());
            }
            other => panic!("unexpected native fetch: {other:?}"),
        }
    }

    #[test]
    fn foreign_builtin_lowering_rejects_malformed_hash_empty_candidates_and_digest_domain_substitution() {
        let mut malformed_hash = download_graph();
        let malformed_leaf = malformed_hash.nodes.iter_mut().find(|node| node.node_id == "leaf").expect("leaf");
        malformed_leaf.fixed_output.as_mut().expect("fixed output").digest = "not-hex".to_string();
        malformed_leaf.outputs.get_mut("out").expect("out output").hash = Some("not-hex".to_string());
        assert_class(
            compile_foreign_graph(&malformed_hash, TARGET_PREFIX),
            "foreign-compiler-fixed-output-hash-invalid",
        );

        let mut empty_candidates = download_graph();
        let empty_leaf = empty_candidates.nodes.iter_mut().find(|node| node.node_id == "leaf").expect("leaf");
        empty_leaf.env.remove("url");
        assert_class(
            compile_foreign_graph(&empty_candidates, TARGET_PREFIX),
            "foreign-compiler-fetch-candidate-count-invalid",
        );

        let mut wrong_domain = download_graph();
        wrong_domain
            .nodes
            .iter_mut()
            .find(|node| node.node_id == "leaf")
            .expect("leaf")
            .fixed_output
            .as_mut()
            .expect("fixed output")
            .algorithm = BLAKE3_ALGORITHM.to_string();
        assert_class(compile_foreign_graph(&wrong_domain, TARGET_PREFIX), "foreign-compiler-digest-domain-mismatch");
    }

    #[test]
    fn execution_profile_changes_target_identity_and_reserved_collision_fails() {
        let graph = diamond_graph();
        let first_profile = ExecutionProfile::foreign_nix();
        let mut second_profile = first_profile.clone();
        second_profile.work_directory = "work".to_string();
        second_profile.writable_prefixes = vec!["work".to_string()];

        let first = compile_foreign_graph_with_profile(&graph, TARGET_PREFIX, &first_profile).unwrap();
        let second = compile_foreign_graph_with_profile(&graph, TARGET_PREFIX, &second_profile).unwrap();

        assert_ne!(first.roots, second.roots);
        assert_ne!(first.units[0].execution_profile_digest_blake3, second.units[0].execution_profile_digest_blake3);
        assert_eq!(
            first.units[0]
                .derivation
                .environment
                .get(EXECUTION_PROFILE_BINDING_ENV)
                .map(|value| String::from_utf8_lossy(value).into_owned()),
            Some(first.units[0].execution_profile_digest_blake3.clone())
        );

        let mut collision = diamond_graph();
        collision
            .nodes
            .iter_mut()
            .find(|node| node.node_id == "root")
            .unwrap()
            .env
            .insert(EXECUTION_PROFILE_BINDING_ENV.to_string(), "foreign-value".to_string());
        assert_class(
            compile_foreign_graph_with_profile(&collision, TARGET_PREFIX, &first_profile),
            "foreign-compiler-execution-profile-binding-invalid",
        );
    }

    #[test]
    fn dependency_order_rejects_cycles_duplicate_edges_unknown_outputs_and_partial_coverage() {
        let mut cycle = diamond_graph();
        cycle
            .nodes
            .iter_mut()
            .find(|node| node.node_id == "leaf")
            .expect("leaf")
            .input_derivations
            .push(edge("root", "out"));
        assert_class(foreign_dependency_order(&cycle), "foreign-compiler-cycle");

        let mut duplicate = diamond_graph();
        let root = duplicate.nodes.iter_mut().find(|node| node.node_id == "root").expect("root");
        root.input_derivations.push(edge("left", "out"));
        assert_class(foreign_dependency_order(&duplicate), "foreign-compiler-duplicate-edge");

        let mut unknown_output = diamond_graph();
        unknown_output.nodes.iter_mut().find(|node| node.node_id == "root").expect("root").input_derivations[0]
            .output_name = "missing".to_string();
        assert_class(foreign_dependency_order(&unknown_output), "foreign-compiler-unknown-output");

        let mut missing_node = diamond_graph();
        missing_node.nodes.iter_mut().find(|node| node.node_id == "root").expect("root").input_derivations[0].node_id =
            "absent".to_string();
        assert_class(foreign_dependency_order(&missing_node), "foreign-compiler-missing-node");

        let reachable = ["leaf".to_string(), "root".to_string()].into_iter().collect::<BTreeSet<_>>();
        assert_class(validate_order_coverage(&reachable, &["leaf".to_string()]), "foreign-compiler-partial-plan");
    }

    #[test]
    fn compiler_rejects_duplicate_maps_output_drift_and_unknown_embedded_paths() {
        let mut duplicate_map = diamond_graph();
        let repeated_path = foreign_output("left", "out");
        duplicate_map
            .nodes
            .iter_mut()
            .find(|node| node.node_id == "right")
            .expect("right")
            .outputs
            .get_mut("out")
            .expect("right output")
            .path = repeated_path;
        assert_class(compile_foreign_graph(&duplicate_map, TARGET_PREFIX), "foreign-compiler-duplicate-map-entry");

        let mut output_drift = diamond_graph();
        let leaf = output_drift.nodes.iter_mut().find(|node| node.node_id == "leaf").expect("leaf");
        leaf.outputs.insert("dev".to_string(), OutputDeclaration {
            path: foreign_output("leaf", "dev"),
            hash: None,
        });
        assert_class(compile_foreign_graph(&output_drift, TARGET_PREFIX), "foreign-compiler-output-name-drift");

        let mut unknown = diamond_graph();
        unknown
            .nodes
            .iter_mut()
            .find(|node| node.node_id == "root")
            .expect("root")
            .args
            .push(format!("{SOURCE_PREFIX}/99999999999999999999999999999999-unknown/bin/tool"));
        assert_class(compile_foreign_graph(&unknown, TARGET_PREFIX), "foreign-compiler-unknown-reference");

        let mut unsupported_builtin = diamond_graph();
        unsupported_builtin.nodes.iter_mut().find(|node| node.node_id == "root").expect("root").builtin =
            "foreign.unsupported".to_string();
        assert_class(
            compile_foreign_graph(&unsupported_builtin, TARGET_PREFIX),
            "foreign-compiler-unsupported-builtin",
        );
    }

    fn diamond_graph() -> ForeignDerivationGraph {
        let leaf = fixed_node("leaf");
        let mut left = plain_node("left", vec![edge("leaf", "out")]);
        left.env.insert("leafPath".to_string(), foreign_output("leaf", "out"));
        left.outputs.insert("dev".to_string(), OutputDeclaration {
            path: foreign_output("left", "dev"),
            hash: None,
        });
        let mut right = plain_node("right", vec![edge("leaf", "out")]);
        right.env.insert("leafPath".to_string(), foreign_output("leaf", "out"));
        let mut root = plain_node("root", vec![edge("left", "out"), edge("left", "dev"), edge("right", "out")]);
        root.env.insert("leftPath".to_string(), format!("{}/bin/tool", foreign_output("left", "out")));
        root.env.insert("leftDevPath".to_string(), format!("{}/include", foreign_output("left", "dev")));
        ForeignDerivationGraph {
            schema: "foreign-derivation-graph-v1".to_string(),
            producer: ProducerSummary {
                kind: "compiler-test".to_string(),
                identity: "compiler-test-producer".to_string(),
                revision: "test-revision".to_string(),
            },
            source_store_prefixes: vec![SOURCE_PREFIX.to_string()],
            target_store_prefix: None,
            root_derivation_ids: vec!["root".to_string()],
            nodes: vec![root, right, leaf, left],
            source_payloads: Vec::new(),
            unsupported_features: Vec::<UnsupportedFeature>::new(),
            frontend_metadata: Vec::<FrontendMetadata>::new(),
            hash_domains: Vec::<HashDomainRecord>::new(),
        }
    }

    fn download_graph() -> ForeignDerivationGraph {
        let mut graph = diamond_graph();
        let leaf = graph.nodes.iter_mut().find(|node| node.node_id == "leaf").expect("leaf");
        leaf.builtin = FOREIGN_DOWNLOAD_BUILTIN.to_string();
        leaf.env.insert("url".to_string(), "https://primary.example/source.tar".to_string());
        leaf.env.insert("mode".to_string(), FETCH_MODE_RECURSIVE.to_string());
        leaf.env.insert("executable".to_string(), "0".to_string());
        graph
    }

    fn add_fetch_payload(
        graph: &mut ForeignDerivationGraph,
        payload_id: &str,
        source_name: &str,
        mirrors: Vec<String>,
    ) {
        graph.source_payloads.push(SourcePayload {
            payload_id: payload_id.to_string(),
            kind: "store-path".to_string(),
            content_ref: foreign_source(source_name),
            embedded_text: None,
            mirrors,
        });
    }

    fn fixed_node(node_id: &str) -> ForeignDerivationNode {
        let mut node = plain_node(node_id, Vec::new());
        node.fixed_output = Some(FixedOutputMetadata {
            algorithm: "sha256".to_string(),
            digest: EMPTY_HASH.to_string(),
            recursive: true,
        });
        node.builtin = FIXED_OUTPUT_FETCH_BUILTIN.to_string();
        node.outputs.get_mut("out").expect("out output").hash = Some(EMPTY_HASH.to_string());
        node
    }

    fn plain_node(node_id: &str, input_derivations: Vec<InputDerivationEdge>) -> ForeignDerivationNode {
        let output_path = foreign_output(node_id, "out");
        ForeignDerivationNode {
            node_id: node_id.to_string(),
            original_derivation: foreign_derivation(node_id),
            name: node_id.to_string(),
            system: "x86_64-linux".to_string(),
            builder: "/bin/sh".to_string(),
            args: vec!["-c".to_string(), "true".to_string()],
            env: BTreeMap::from([
                ("name".to_string(), node_id.to_string()),
                ("out".to_string(), output_path.clone()),
                ("system".to_string(), "x86_64-linux".to_string()),
            ]),
            outputs: BTreeMap::from([("out".to_string(), OutputDeclaration {
                path: output_path,
                hash: None,
            })]),
            input_derivations,
            source_refs: Vec::new(),
            fixed_output: None,
            builtin: "nix.derivation".to_string(),
            declared_references: Vec::new(),
            sandbox_capabilities: Vec::new(),
            unsupported_features: Vec::new(),
            cache_hints: Vec::<CacheHint>::new(),
        }
    }

    fn edge(node_id: &str, output_name: &str) -> InputDerivationEdge {
        InputDerivationEdge {
            node_id: node_id.to_string(),
            output_name: output_name.to_string(),
        }
    }

    fn foreign_derivation(name: &str) -> String {
        format!("{SOURCE_PREFIX}/11111111111111111111111111111111-{name}.drv")
    }

    fn foreign_output(name: &str, output: &str) -> String {
        format!("{SOURCE_PREFIX}/22222222222222222222222222222222-{name}-{output}")
    }

    fn foreign_source(name: &str) -> String {
        format!("{SOURCE_PREFIX}/33333333333333333333333333333333-{name}")
    }

    fn assert_class<T>(result: Result<T, ImportDiagnostic>, expected: &str) {
        let error = result.err().expect("fixture must be rejected");
        assert_eq!(error.class, expected);
    }
}
