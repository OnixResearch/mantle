use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde_json::Value;

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
pub fn bind_octet_job(mut handoff: ObservedHandoff, binding: ProducerBinding) -> Result<BoundProducerHandoff, String> {
    if handoff.observation.external_report_identity.is_none() {
        return Err("producer job binding requires the exact external verification report".to_string());
    }
    let job = document(&binding.job)?;
    check_producer_job_identity(&job, &handoff)?;
    let receipt = document(&handoff.receipt)?;
    check_producer_job_rail(&job, &receipt)?;
    if job.pointer("/execution/network_allowed").and_then(Value::as_bool) != Some(false) {
        return Err("selected producer job does not declare denied-network execution".to_string());
    }
    if binding.graph.is_empty() || binding.graph.len() > MAX_TOOL_GRAPH_BYTES {
        return Err("tool graph byte bound exceeded or empty".to_string());
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
        .map_err(|error| error.to_string())?,
    );
    handoff.observation.producer_job_identity = Some(job_identity);
    handoff.observation.tool_graph_identity = Some(tool_graph_identity);
    Ok(BoundProducerHandoff { handoff, binding })
}

/// The producer job must name the selected profile and provider.
fn check_producer_job_identity(job: &Value, handoff: &ObservedHandoff) -> Result<(), String> {
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
fn check_producer_job_rail(job: &Value, receipt: &Value) -> Result<(), String> {
    equal(text(job, "/execution/rail_id")?, Expected(text(receipt, "/rail_id")?), "job rail")?;
    Ok(())
}

/// The tool graph must be a bounded Nix derivation closure rooted at the
/// selected root derivation, with every dependency inside the closure.
fn check_tool_graph(graph_bytes: &[u8], root_derivation: &str) -> Result<(), String> {
    let graph: Value = serde_json::from_slice(graph_bytes).map_err(|error| format!("tool graph JSON: {error}"))?;
    let nodes = graph.as_object().ok_or("tool graph must be Nix derivation-show JSON")?;
    if nodes.is_empty() || nodes.len() > MAX_TOOL_DERIVATIONS || !nodes.contains_key(root_derivation) {
        return Err("tool graph root absent or derivation bound exceeded".to_string());
    }
    for (path, node) in nodes {
        if !path.starts_with("/nix/store/") || !path.ends_with(".drv") {
            return Err("invalid producer derivation identity".to_string());
        }
        let dependencies = node.get("inputDrvs").and_then(Value::as_object).ok_or("missing tool inputDrvs")?;
        if dependencies.keys().any(|dependency| !nodes.contains_key(dependency)) {
            return Err("producer tool dependency closure is incomplete".to_string());
        }
        let sources = node.get("inputSrcs").and_then(Value::as_array).ok_or("missing tool inputSrcs")?;
        if sources.iter().any(|source| !source.as_str().is_some_and(|path| path.starts_with("/nix/store/"))) {
            return Err("producer graph contains a non-store source input".to_string());
        }
    }
    debug_assert!(nodes.len() <= MAX_TOOL_DERIVATIONS);
    debug_assert!(nodes.contains_key(root_derivation));
    Ok(())
}
