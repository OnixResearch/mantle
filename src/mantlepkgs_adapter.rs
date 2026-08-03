// r[impl mantlepkgs.unsupported_package_diagnostics]
// r[impl mantlepkgs.recomputed_rebuild]
// r[verify mantlepkgs.unsupported_package_diagnostics]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use mantlepkgs_core::CatalogBlocker;
use mantlepkgs_core::ForeignNodeFact;
use mantlepkgs_core::PackageGraphObservation;
use mantlepkgs_core::PackageSelector;
use mantlepkgs_core::SourceRequirementFact;

use crate::foreign_derivation_import::ForeignDerivationGraph;
use crate::foreign_derivation_import::ForeignDerivationNode;
use crate::foreign_derivation_import::FrontendMetadata;
use crate::foreign_derivation_import::HashDomainRecord;
use crate::foreign_derivation_import::PackageIndex;
use crate::foreign_derivation_import::PackageIndexEntry;
use crate::foreign_derivation_import::SourcePayload;
use crate::foreign_derivation_import::UnsupportedFeature;
use crate::foreign_graph_compiler::compile_foreign_graph;

const NIX_STORE_OBJECT_PREFIX: &str = "/nix/store/";
const MAX_OPAQUE_SOURCE_SCAN_BYTES: usize = 16_777_216;

#[derive(Clone)]
pub(crate) struct ProducedPackageFacts {
    pub(crate) selector_name: String,
    pub(crate) system: String,
    pub(crate) expected_graph_digest_blake3: String,
    pub(crate) expected_index_digest_blake3: String,
    pub(crate) graph: Option<ForeignDerivationGraph>,
    pub(crate) package_index: Option<PackageIndex>,
    pub(crate) producer_blockers: Vec<CatalogBlocker>,
}

#[derive(Clone)]
pub(crate) struct MergedForeignArtifacts {
    pub(crate) graph: ForeignDerivationGraph,
    pub(crate) package_index: PackageIndex,
}

pub(crate) fn observe_package(
    selector: &PackageSelector,
    produced: &ProducedPackageFacts,
    target_store_prefix: &str,
) -> PackageGraphObservation {
    let mut blockers = produced.producer_blockers.clone();
    let Some(graph) = produced.graph.as_ref() else {
        return missing_artifact_observation(selector, produced, blockers, "missing-producer-graph");
    };
    let Some(index) = produced.package_index.as_ref() else {
        return missing_artifact_observation(selector, produced, blockers, "missing-producer-index");
    };
    let observed_graph_digest = canonical_digest(graph).unwrap_or_default();
    let observed_index_digest = canonical_digest(index).unwrap_or_default();
    let root_node_id = selected_root(index, selector, &mut blockers).unwrap_or_default();
    collect_declared_blockers(graph, index, selector, &mut blockers);
    let source_requirements = match compile_foreign_graph(graph, target_store_prefix) {
        Ok(compiled) => compiled
            .source_requirements
            .iter()
            .map(|requirement| SourceRequirementFact {
                identity: requirement.payload_id.clone(),
                descriptor_digest_blake3: requirement.descriptor_digest.clone(),
            })
            .collect(),
        Err(diagnostic) => {
            blockers.push(CatalogBlocker::new(
                &diagnostic.class,
                diagnostic.node_id.as_deref().unwrap_or(&selector.name),
                &diagnostic.message,
            ));
            Vec::new()
        }
    };
    let nodes = graph
        .nodes
        .iter()
        .map(|node| ForeignNodeFact {
            foreign_identity: node.original_derivation.clone(),
            node_id: node.node_id.clone(),
            canonical_digest_blake3: canonical_digest(node).unwrap_or_default(),
        })
        .collect();
    blockers.sort();
    blockers.dedup();
    PackageGraphObservation {
        selector_name: produced.selector_name.clone(),
        system: produced.system.clone(),
        root_node_id,
        producer_graph_digest_blake3: produced.expected_graph_digest_blake3.clone(),
        observed_graph_digest_blake3: observed_graph_digest,
        producer_index_digest_blake3: produced.expected_index_digest_blake3.clone(),
        observed_index_digest_blake3: observed_index_digest,
        nodes,
        source_requirements,
        blockers,
    }
}

pub(crate) fn merge_buildable_packages(
    packages: &[ProducedPackageFacts],
    selectors: &[PackageSelector],
) -> Result<MergedForeignArtifacts, CatalogBlocker> {
    assert!(!packages.is_empty(), "a buildable package batch must not be empty");
    let selector_map = selectors
        .iter()
        .map(|selector| ((selector.system.as_str(), selector.name.as_str()), selector))
        .collect::<BTreeMap<_, _>>();
    let mut roots = BTreeSet::new();
    let mut nodes = BTreeMap::<String, ForeignDerivationNode>::new();
    let mut payloads = BTreeMap::<String, SourcePayload>::new();
    let mut unsupported = BTreeSet::<UnsupportedFeature>::new();
    let mut frontend = BTreeSet::<FrontendMetadata>::new();
    let mut hash_domains = BTreeSet::<HashDomainRecord>::new();
    let mut entries = Vec::new();
    for package in packages {
        let graph = package.graph.as_ref().ok_or_else(|| {
            CatalogBlocker::new("missing-producer-graph", &package.selector_name, "the producer graph is missing")
        })?;
        let index = package.package_index.as_ref().ok_or_else(|| {
            CatalogBlocker::new("missing-producer-index", &package.selector_name, "the producer index is missing")
        })?;
        let selector = selector_map
            .get(&(package.system.as_str(), package.selector_name.as_str()))
            .copied()
            .ok_or_else(|| {
                CatalogBlocker::new(
                    "unexpected-producer-selection",
                    &package.selector_name,
                    "the producer package has no manifest selector",
                )
            })?;
        merge_graph(graph, &mut roots, &mut nodes, &mut payloads, &mut unsupported, &mut frontend, &mut hash_domains)?;
        entries.push(merged_index_entry(index, selector)?);
    }
    entries.sort();
    let graph = ForeignDerivationGraph {
        schema: packages[0].graph.as_ref().expect("first buildable graph exists").schema.clone(),
        producer: packages[0].graph.as_ref().expect("first buildable graph exists").producer.clone(),
        source_store_prefixes: vec![crate::foreign_derivation_import::NIX_SOURCE_PREFIX.into()],
        target_store_prefix: None,
        root_derivation_ids: roots.into_iter().collect(),
        nodes: nodes.into_values().collect(),
        source_payloads: payloads.into_values().collect(),
        unsupported_features: unsupported.into_iter().collect(),
        frontend_metadata: frontend.into_iter().collect(),
        hash_domains: hash_domains.into_iter().collect(),
    };
    let package_index = PackageIndex {
        schema: packages[0].package_index.as_ref().expect("first buildable index exists").schema.clone(),
        entries,
    };
    Ok(MergedForeignArtifacts { graph, package_index })
}

fn missing_artifact_observation(
    selector: &PackageSelector,
    produced: &ProducedPackageFacts,
    mut blockers: Vec<CatalogBlocker>,
    code: &str,
) -> PackageGraphObservation {
    blockers.push(CatalogBlocker::new(code, &selector.name, "the producer did not emit the required artifact"));
    blockers.sort();
    blockers.dedup();
    PackageGraphObservation {
        selector_name: produced.selector_name.clone(),
        system: produced.system.clone(),
        root_node_id: String::new(),
        producer_graph_digest_blake3: produced.expected_graph_digest_blake3.clone(),
        observed_graph_digest_blake3: String::new(),
        producer_index_digest_blake3: produced.expected_index_digest_blake3.clone(),
        observed_index_digest_blake3: String::new(),
        nodes: Vec::new(),
        source_requirements: Vec::new(),
        blockers,
    }
}

fn selected_root(
    index: &PackageIndex,
    selector: &PackageSelector,
    blockers: &mut Vec<CatalogBlocker>,
) -> Option<String> {
    let matches = index
        .entries
        .iter()
        .filter(|entry| entry.system == selector.system && entry.name == selector.name)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [entry] => Some(entry.root_derivation_id.clone()),
        [] => {
            blockers.push(CatalogBlocker::new(
                "missing-package-index-entry",
                &selector.name,
                "the producer index has no exact package entry",
            ));
            None
        }
        _ => {
            blockers.push(CatalogBlocker::new(
                "ambiguous-package-index-entry",
                &selector.name,
                "the producer index has more than one exact package entry",
            ));
            None
        }
    }
}

fn collect_declared_blockers(
    graph: &ForeignDerivationGraph,
    index: &PackageIndex,
    selector: &PackageSelector,
    blockers: &mut Vec<CatalogBlocker>,
) {
    for feature in &graph.unsupported_features {
        if feature.mandatory {
            blockers.push(CatalogBlocker::new(&feature.class, &selector.name, &feature.message));
        }
    }
    for node in &graph.nodes {
        for feature in &node.unsupported_features {
            if feature.mandatory {
                blockers.push(CatalogBlocker::new(&feature.class, &node.node_id, &feature.message));
            }
        }
    }
    for entry in &index.entries {
        for class in &entry.unsupported_metadata_classes {
            blockers.push(CatalogBlocker::new(
                "unsupported-frontend-metadata",
                &entry.name,
                &format!("unsupported frontend metadata class: {class}"),
            ));
        }
    }
    for payload in &graph.source_payloads {
        if payload
            .embedded_text
            .as_ref()
            .is_some_and(|text| text.len() > MAX_OPAQUE_SOURCE_SCAN_BYTES || text.contains(NIX_STORE_OBJECT_PREFIX))
        {
            blockers.push(CatalogBlocker::new(
                "hard-coded-source-store-assumption",
                &payload.payload_id,
                "an opaque embedded source payload contains a Nix store path or exceeds the scan limit",
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn merge_graph(
    graph: &ForeignDerivationGraph,
    roots: &mut BTreeSet<String>,
    nodes: &mut BTreeMap<String, ForeignDerivationNode>,
    payloads: &mut BTreeMap<String, SourcePayload>,
    unsupported: &mut BTreeSet<UnsupportedFeature>,
    frontend: &mut BTreeSet<FrontendMetadata>,
    hash_domains: &mut BTreeSet<HashDomainRecord>,
) -> Result<(), CatalogBlocker> {
    roots.extend(graph.root_derivation_ids.iter().cloned());
    unsupported.extend(graph.unsupported_features.iter().cloned());
    frontend.extend(graph.frontend_metadata.iter().cloned());
    hash_domains.extend(graph.hash_domains.iter().cloned());
    for node in &graph.nodes {
        match nodes.get(&node.original_derivation) {
            Some(previous) if previous != node => {
                return Err(CatalogBlocker::new(
                    "conflicting-foreign-node",
                    &node.original_derivation,
                    "the same foreign derivation has conflicting graph facts",
                ));
            }
            Some(_) => {}
            None => {
                nodes.insert(node.original_derivation.clone(), node.clone());
            }
        }
    }
    for payload in &graph.source_payloads {
        match payloads.get(&payload.payload_id) {
            Some(previous) if previous != payload => {
                return Err(CatalogBlocker::new(
                    "conflicting-source-requirement",
                    &payload.payload_id,
                    "the same source payload identity has conflicting facts",
                ));
            }
            Some(_) => {}
            None => {
                payloads.insert(payload.payload_id.clone(), payload.clone());
            }
        }
    }
    Ok(())
}

fn merged_index_entry(index: &PackageIndex, selector: &PackageSelector) -> Result<PackageIndexEntry, CatalogBlocker> {
    let mut matches =
        index.entries.iter().filter(|entry| entry.name == selector.name && entry.system == selector.system);
    let source = matches.next().ok_or_else(|| {
        CatalogBlocker::new(
            "missing-package-index-entry",
            &selector.name,
            "the producer index has no exact package entry",
        )
    })?;
    if matches.next().is_some() {
        return Err(CatalogBlocker::new(
            "ambiguous-package-index-entry",
            &selector.name,
            "the producer index has more than one exact package entry",
        ));
    }
    let mut entry = source.clone();
    entry.aliases = selector.aliases.clone();
    Ok(entry)
}

fn canonical_digest<T: serde::Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_vec(value).map(|bytes| blake3::hash(&bytes).to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use mantlepkgs_core::PackageDisposition;

    use super::*;

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn selector() -> PackageSelector {
        PackageSelector {
            name: "hello".into(),
            attribute: "hello".into(),
            system: "x86_64-linux".into(),
            aliases: vec!["hi".into()],
        }
    }

    #[test]
    fn supported_foreign_graph_produces_buildable_observation() {
        let (graph, package_index) = crate::foreign_derivation_import::nix_like_hello_fixture();
        let facts = ProducedPackageFacts {
            selector_name: "hello".into(),
            system: "x86_64-linux".into(),
            expected_graph_digest_blake3: canonical_digest(&graph).unwrap(),
            expected_index_digest_blake3: canonical_digest(&package_index).unwrap(),
            graph: Some(graph),
            package_index: Some(package_index),
            producer_blockers: Vec::new(),
        };

        let observation = observe_package(&selector(), &facts, "/mantle/store");
        assert!(observation.blockers.is_empty());
        assert!(!observation.nodes.is_empty());
        assert_eq!(observation.producer_graph_digest_blake3, observation.observed_graph_digest_blake3);
    }

    #[test]
    fn graph_cycle_and_opaque_store_path_remain_blockers() {
        let (mut graph, package_index) = crate::foreign_derivation_import::nix_like_hello_fixture();
        let root = graph.root_derivation_ids[0].clone();
        graph.nodes[0].input_derivations.push(crate::foreign_derivation_import::InputDerivationEdge {
            node_id: root,
            output_name: "out".into(),
        });
        graph.source_payloads[0].embedded_text = Some("opaque /nix/store/hidden-reference".into());
        let facts = ProducedPackageFacts {
            selector_name: "hello".into(),
            system: "x86_64-linux".into(),
            expected_graph_digest_blake3: canonical_digest(&graph).unwrap(),
            expected_index_digest_blake3: canonical_digest(&package_index).unwrap(),
            graph: Some(graph),
            package_index: Some(package_index),
            producer_blockers: Vec::new(),
        };

        let observation = observe_package(&selector(), &facts, "/mantle/store");
        let codes = observation.blockers.iter().map(|item| item.code.as_str()).collect::<BTreeSet<_>>();
        assert!(codes.contains("foreign-compiler-cycle"));
        assert!(codes.contains("hard-coded-source-store-assumption"));
    }

    #[test]
    fn stale_digest_and_unsupported_builtin_remain_blockers() {
        let (mut graph, package_index) = crate::foreign_derivation_import::nix_like_hello_fixture();
        graph.nodes[0].builtin = "unsupported:test".into();
        let facts = ProducedPackageFacts {
            selector_name: "hello".into(),
            system: "x86_64-linux".into(),
            expected_graph_digest_blake3: DIGEST.into(),
            expected_index_digest_blake3: canonical_digest(&package_index).unwrap(),
            graph: Some(graph),
            package_index: Some(package_index),
            producer_blockers: Vec::new(),
        };

        let observation = observe_package(&selector(), &facts, "/mantle/store");
        assert!(observation.blockers.iter().any(|item| item.code.contains("builtin")));
        assert_ne!(observation.producer_graph_digest_blake3, observation.observed_graph_digest_blake3);
        let disposition = PackageDisposition::Blocked {
            blockers: observation.blockers,
        };
        assert!(matches!(disposition, PackageDisposition::Blocked { .. }));
    }
}
