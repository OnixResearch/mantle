use alloc::string::String;
use alloc::vec::Vec;

use serde_json::Value;

use crate::BridgeError;
use crate::Expected;
use crate::ObservedHandoff;
use crate::digest;
use crate::document;
use crate::equal;
use crate::text;

pub const MAX_TOOL_GRAPH_BYTES: usize = 268_435_456;
pub const MAX_TOOL_DERIVATIONS: usize = 32_768;

pub struct ProducerBinding {
    pub job: Vec<u8>,
    pub graph: Vec<u8>,
    pub flake_metadata: Vec<u8>,
    pub flake_lock: Vec<u8>,
    pub root_derivation: String,
}
pub struct BoundProducerHandoff {
    pub handoff: ObservedHandoff,
    pub binding: ProducerBinding,
}

/// Consume the externally authored job plus Nix's complete ordinary derivation
/// graph. No action is executed and no proof policy is inferred from the graph.
pub fn bind_octet_job(
    mut handoff: ObservedHandoff,
    binding: ProducerBinding,
) -> Result<BoundProducerHandoff, BridgeError> {
    if handoff.observation.external_report_identity.is_none() {
        return Err(BridgeError::ProducerJobRequiresVerification);
    }
    let job = document(&binding.job)?;
    check_producer_job_identity(&job, &handoff)?;
    let receipt = document(&handoff.receipt)?;
    check_producer_job_rail(&job, &receipt)?;
    if job.pointer("/execution/network_allowed").and_then(Value::as_bool) != Some(false) {
        return Err(BridgeError::ProducerNetworkNotDenied);
    }
    if binding.graph.is_empty() || binding.graph.len() > MAX_TOOL_GRAPH_BYTES {
        return Err(BridgeError::ToolGraphByteBound);
    }
    let metadata = document(&binding.flake_metadata)?;
    equal(
        text(&metadata, "/revision")?,
        Expected(&handoff.observation.packaging.source_revision),
        "producer source revision",
    )?;
    let _lock = document(&binding.flake_lock)?;
    check_tool_graph(&binding.graph, &binding.root_derivation)?;
    let job_identity = digest(&binding.job);
    let tool_graph_identity = digest(&binding.graph);
    debug_assert!(!job_identity.is_empty());
    debug_assert!(!tool_graph_identity.is_empty());
    handoff.observation.association_identity = digest(
        &serde_json::to_vec(&(
            "mantle.external-producer-job-association.v1",
            &handoff.observation.association_identity,
            &job_identity,
            &tool_graph_identity,
            &binding.root_derivation,
            digest(&binding.flake_metadata),
            digest(&binding.flake_lock),
        ))
        .map_err(BridgeError::Serialization)?,
    );
    handoff.observation.producer_job_identity = Some(job_identity);
    handoff.observation.tool_graph_identity = Some(tool_graph_identity);
    Ok(BoundProducerHandoff { handoff, binding })
}

/// The producer job must name the selected profile and provider.
fn check_producer_job_identity(job: &Value, handoff: &ObservedHandoff) -> Result<(), BridgeError> {
    equal(text(job, "/schema_version")?, Expected("octet-mantle-producer-job/v1"), "producer job schema")?;
    equal(text(job, "/owner")?, Expected("Octet"), "producer owner")?;
    equal(
        text(job, "/profile/identity")?,
        Expected(&handoff.observation.packaging.profile_identity),
        "job profile",
    )?;
    equal(
        text(job, "/provider/identity")?,
        Expected(&handoff.observation.packaging.provider_identity),
        "job provider",
    )?;
    debug_assert!(job.get("schema_version").is_some());
    debug_assert!(job.get("owner").is_some());
    Ok(())
}

/// The producer job must run on the same rail as the observed receipt.
fn check_producer_job_rail(job: &Value, receipt: &Value) -> Result<(), BridgeError> {
    equal(text(job, "/execution/rail_id")?, Expected(text(receipt, "/rail_id")?), "job rail")?;
    Ok(())
}

/// The tool graph must be a bounded Nix derivation closure rooted at the
/// selected root derivation, with every dependency inside the closure.
///
/// Exactly two wire shapes are accepted: Nix's legacy `derivation show` map and
/// the versioned (`version: 4`) map that Nix 2.36 and the producer export emit.
// r[impl verification_bridge.octet_producer_layout]
fn check_tool_graph(graph_bytes: &[u8], root_derivation: &str) -> Result<(), BridgeError> {
    let graph: Value = serde_json::from_slice(graph_bytes).map_err(BridgeError::ToolGraphJson)?;
    graph.as_object().ok_or(BridgeError::InvalidToolGraphShape)?;
    let versioned = graph.get("version").is_some();
    if versioned {
        check_versioned_tool_graph(&graph, root_derivation)
    } else {
        check_legacy_tool_graph(&graph, root_derivation)
    }
}

/// The versioned map keys nodes by store basename under `derivations`.
fn check_versioned_tool_graph(graph: &Value, root_derivation: &str) -> Result<(), BridgeError> {
    if graph.get("version").and_then(Value::as_u64) != Some(4) {
        return Err(BridgeError::UnsupportedToolGraphVersion);
    }
    let nodes = graph.get("derivations").and_then(Value::as_object).ok_or(BridgeError::InvalidToolGraphShape)?;
    let root = final_component(root_derivation);
    if nodes.is_empty() || nodes.len() > MAX_TOOL_DERIVATIONS || !nodes.contains_key(root) {
        return Err(BridgeError::ToolGraphRootOrCountBound);
    }
    for (name, node) in nodes {
        check_versioned_node(nodes, name, node)?;
    }
    debug_assert!(nodes.len() <= MAX_TOOL_DERIVATIONS);
    debug_assert!(nodes.contains_key(root));
    Ok(())
}

/// One versioned node carries `version: 4`, `inputs.drvs` and `inputs.srcs`.
fn check_versioned_node(nodes: &serde_json::Map<String, Value>, name: &str, node: &Value) -> Result<(), BridgeError> {
    if !is_store_basename(name) || !name.ends_with(".drv") {
        return Err(BridgeError::InvalidProducerDerivation);
    }
    let node = node.as_object().ok_or(BridgeError::InvalidProducerDerivation)?;
    if node.contains_key("inputDrvs") || node.contains_key("inputSrcs") {
        return Err(BridgeError::MixedToolGraphShape);
    }
    if node.get("version").and_then(Value::as_u64) != Some(4) {
        return Err(BridgeError::InvalidProducerDerivation);
    }
    let inputs = node.get("inputs").and_then(Value::as_object).ok_or(BridgeError::MissingToolInputDerivations)?;
    let dependencies = inputs.get("drvs").and_then(Value::as_object).ok_or(BridgeError::MissingToolInputDerivations)?;
    let sources = inputs.get("srcs").and_then(Value::as_array).ok_or(BridgeError::MissingToolInputSources)?;
    if dependencies.len() > MAX_TOOL_DERIVATIONS || sources.len() > MAX_TOOL_DERIVATIONS {
        return Err(BridgeError::ToolGraphRootOrCountBound);
    }
    for (dependency, selected) in dependencies {
        if !is_store_basename(dependency) || !dependency.ends_with(".drv") {
            return Err(BridgeError::InvalidProducerDerivation);
        }
        if !nodes.contains_key(dependency) {
            return Err(BridgeError::IncompleteToolClosure);
        }
        check_versioned_selection(selected)?;
    }
    for source in sources {
        if !source.as_str().is_some_and(is_store_basename) {
            return Err(BridgeError::NonStoreToolSource);
        }
    }
    debug_assert!(dependencies.len() <= MAX_TOOL_DERIVATIONS);
    debug_assert!(sources.len() <= MAX_TOOL_DERIVATIONS);
    Ok(())
}

/// One versioned input selection names static outputs and no dynamic output.
fn check_versioned_selection(selected: &Value) -> Result<(), BridgeError> {
    let selected = selected.as_object().ok_or(BridgeError::InvalidProducerDerivation)?;
    let dynamic = selected.get("dynamicOutputs").and_then(Value::as_object);
    if !dynamic.is_some_and(serde_json::Map::is_empty) {
        return Err(BridgeError::NonEmptyDynamicToolOutputs);
    }
    let outputs = selected.get("outputs").and_then(Value::as_array).ok_or(BridgeError::InvalidProducerDerivation)?;
    if outputs.iter().any(|output| !output.is_string()) {
        return Err(BridgeError::InvalidProducerDerivation);
    }
    debug_assert!(dynamic.is_some_and(serde_json::Map::is_empty));
    debug_assert!(outputs.iter().all(Value::is_string));
    Ok(())
}

/// The legacy map keys nodes by full store path and never carries `version` or
/// `inputs`; a node with either is a mixed shape and fails closed.
fn check_legacy_tool_graph(graph: &Value, root_derivation: &str) -> Result<(), BridgeError> {
    let nodes = graph.as_object().ok_or(BridgeError::InvalidToolGraphShape)?;
    if nodes.is_empty() || nodes.len() > MAX_TOOL_DERIVATIONS || !nodes.contains_key(root_derivation) {
        return Err(BridgeError::ToolGraphRootOrCountBound);
    }
    for (path, node) in nodes {
        if !path.starts_with("/nix/store/") || !path.ends_with(".drv") {
            return Err(BridgeError::InvalidProducerDerivation);
        }
        if node.get("version").is_some() || node.get("inputs").is_some() {
            return Err(BridgeError::MixedToolGraphShape);
        }
        let dependencies =
            node.get("inputDrvs").and_then(Value::as_object).ok_or(BridgeError::MissingToolInputDerivations)?;
        if dependencies.keys().any(|dependency| !nodes.contains_key(dependency)) {
            return Err(BridgeError::IncompleteToolClosure);
        }
        let sources = node.get("inputSrcs").and_then(Value::as_array).ok_or(BridgeError::MissingToolInputSources)?;
        if sources.iter().any(|source| !source.as_str().is_some_and(|path| path.starts_with("/nix/store/"))) {
            return Err(BridgeError::NonStoreToolSource);
        }
    }
    debug_assert!(nodes.len() <= MAX_TOOL_DERIVATIONS);
    debug_assert!(nodes.contains_key(root_derivation));
    Ok(())
}

/// The final path component of one store path.
fn final_component(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// True when one string is a bounded Nix store basename (`<hash>-<name>`).
fn is_store_basename(name: &str) -> bool {
    let Some((hash, rest)) = name.split_once('-') else {
        return false;
    };
    let is_hash = hash.len() == 32 && hash.bytes().all(|byte| b"0123456789abcdfghijklmnpqrsvwxyz".contains(&byte));
    let is_name = !rest.is_empty()
        && rest.len() <= 222
        && rest
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'.' | b'_' | b'?' | b'=' | b'-'));
    is_hash && is_name
}

// r[impl verification_bridge.octet_producer_layout]
#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::String;
    use alloc::vec::Vec;

    use serde_json::Value;

    use super::check_tool_graph;
    use crate::BridgeError;

    /// A canonical 32-character Nix base32 run, padded per fixture.
    const HASH: &str = "00000000000000000000000000000000";
    const SOURCE: &str = "11111111111111111111111111111111-source";

    fn derivation(name: &str) -> String {
        format!("{HASH}-{name}.drv")
    }

    fn root_path() -> String {
        format!("/nix/store/{}", derivation("root"))
    }

    fn bytes(graph: &Value) -> Vec<u8> {
        serde_json::to_vec(graph).unwrap()
    }

    fn versioned_graph() -> Value {
        serde_json::json!({
            "version": 4,
            "derivations": {
                (derivation("root")): {
                    "version": 4,
                    "inputs": {
                        "drvs": {
                            (derivation("dependency")): {
                                "outputs": ["out"],
                                "dynamicOutputs": {}
                            }
                        },
                        "srcs": [SOURCE]
                    }
                },
                (derivation("dependency")): {
                    "version": 4,
                    "inputs": {"drvs": {}, "srcs": []}
                }
            }
        })
    }

    fn legacy_graph() -> Value {
        serde_json::json!({
            (format!("/nix/store/{}", derivation("root"))): {
                "inputDrvs": {
                    (format!("/nix/store/{}", derivation("dependency"))): ["out"]
                },
                "inputSrcs": [format!("/nix/store/{SOURCE}")]
            },
            (format!("/nix/store/{}", derivation("dependency"))): {
                "inputDrvs": {},
                "inputSrcs": []
            }
        })
    }

    fn assert_graph_error(graph: &Value, expected: fn(&BridgeError) -> bool) {
        let error = check_tool_graph(&bytes(graph), &root_path()).expect_err("malformed graph admitted");
        assert!(expected(&error), "{error}");
    }

    fn is_mixed(error: &BridgeError) -> bool {
        matches!(error, BridgeError::MixedToolGraphShape)
    }

    fn is_version(error: &BridgeError) -> bool {
        matches!(error, BridgeError::UnsupportedToolGraphVersion)
    }

    fn is_root(error: &BridgeError) -> bool {
        matches!(error, BridgeError::ToolGraphRootOrCountBound)
    }

    fn is_closure(error: &BridgeError) -> bool {
        matches!(error, BridgeError::IncompleteToolClosure)
    }

    fn is_dynamic(error: &BridgeError) -> bool {
        matches!(error, BridgeError::NonEmptyDynamicToolOutputs)
    }

    #[test]
    fn versioned_tool_graph_with_basenames_is_admitted() {
        assert!(check_tool_graph(&bytes(&versioned_graph()), &root_path()).is_ok());
    }

    #[test]
    fn legacy_tool_graph_with_store_paths_is_admitted() {
        assert!(check_tool_graph(&bytes(&legacy_graph()), &root_path()).is_ok());
    }

    #[test]
    fn versioned_node_carrying_legacy_inputs_fails_closed() {
        let mut graph = versioned_graph();
        graph["derivations"][derivation("dependency")]["inputDrvs"] = serde_json::json!({});
        assert_graph_error(&graph, is_mixed);
    }

    #[test]
    fn unsupported_top_level_graph_version_fails_closed() {
        let mut graph = versioned_graph();
        graph["version"] = serde_json::json!(5);
        assert_graph_error(&graph, is_version);
    }

    #[test]
    fn versioned_root_absent_fails_closed() {
        let mut graph = versioned_graph();
        graph["derivations"].as_object_mut().unwrap().remove(&derivation("root"));
        assert_graph_error(&graph, is_root);
    }

    #[test]
    fn versioned_dependency_outside_the_closure_fails_closed() {
        let mut graph = versioned_graph();
        graph["derivations"][derivation("root")]["inputs"]["drvs"][derivation("absent")] =
            serde_json::json!({"outputs": ["out"], "dynamicOutputs": {}});
        assert_graph_error(&graph, is_closure);
    }

    #[test]
    fn versioned_dynamic_output_selection_fails_closed() {
        let mut graph = versioned_graph();
        graph["derivations"][derivation("root")]["inputs"]["drvs"][derivation("dependency")]["dynamicOutputs"] =
            serde_json::json!({"out": "out"});
        assert_graph_error(&graph, is_dynamic);
    }

    #[test]
    fn legacy_node_carrying_versioned_keys_fails_closed() {
        let mut graph = legacy_graph();
        graph[format!("/nix/store/{}", derivation("dependency"))]["version"] = serde_json::json!(4);
        assert_graph_error(&graph, is_mixed);
    }
}
