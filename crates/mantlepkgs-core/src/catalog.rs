// r[impl mantlepkgs.catalog_generation]
// r[impl mantlepkgs.unsupported_package_diagnostics]
// r[impl mantlepkgs.claim_boundary]
// r[verify mantlepkgs.catalog_generation]
// r[verify mantlepkgs.unsupported_package_diagnostics]
// r[verify mantlepkgs.claim_boundary]

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::BLAKE3_HEX_LENGTH;
use crate::CoreFailure;
use crate::Diagnostic;
use crate::MantlepkgsManifest;
use crate::NixpkgsSourceLock;
use crate::PRODUCER_COMMAND_CLASS;
use crate::PackageSelector;
use crate::manifest_digest_blake3;
use crate::normalize_manifest;

pub const CATALOG_PLAN_SCHEMA: &str = "mantlepkgs-catalog-plan-v1";
pub const CATALOG_SCHEMA: &str = "mantlepkgs-catalog-v1";
pub const PRODUCER_RECEIPT_SCHEMA: &str = "mantlepkgs-producer-receipt-v1";
pub const SOURCE_INVENTORY_SCHEMA: &str = "mantlepkgs-source-requirements-v1";
pub const GENERATION_REPORT_SCHEMA: &str = "mantlepkgs-generation-report-v1";
pub const ROLE_SHARED_GRAPH: &str = "shared-foreign-graph";
pub const ROLE_PACKAGE_INDEX: &str = "package-index";
pub const ROLE_SOURCE_INVENTORY: &str = "source-requirement-inventory";
pub const ROLE_TRANSLATION_POLICY: &str = "translation-policy";
pub const ROLE_EXECUTION_PROFILE: &str = "execution-profile";
pub const ROLE_PRODUCER_RECEIPT: &str = "producer-receipt";
pub const SHARED_GRAPH_PATH: &str = "artifacts/shared.graph.json";
pub const PACKAGE_INDEX_PATH: &str = "artifacts/packages.index.json";
pub const SOURCE_INVENTORY_PATH: &str = "artifacts/source-requirements.json";
pub const TRANSLATION_POLICY_PATH: &str = "policies/translation.json";
pub const EXECUTION_PROFILE_PATH: &str = "policies/execution-profile.json";
pub const PRODUCER_RECEIPT_PATH: &str = "producer-receipt.json";
pub const CATALOG_JSON_PATH: &str = "catalog.json";
pub const CATALOG_NICKEL_PATH: &str = "catalog.ncl";
const PLAN_DOMAIN: &[u8] = b"mantle.mantlepkgs.catalog-plan.v1";
const CATALOG_DOMAIN: &[u8] = b"mantle.mantlepkgs.catalog.v1";
const PRODUCER_RECEIPT_DOMAIN: &[u8] = b"mantle.mantlepkgs.producer-receipt.v1";
const DOMAIN_SEPARATOR: u8 = 0;
const MAX_BLOCKERS_PER_PACKAGE: usize = 128;
const MAX_PRODUCER_VERSION_BYTES: usize = 1_024;
const MAX_IMPLEMENTATION_ID_BYTES: usize = 512;
const REQUIRED_ARTIFACT_ROLE_COUNT: usize = 6;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProducerObservation {
    pub command_class: String,
    pub implementation_id: String,
    pub version: String,
    pub executable_digest_blake3: String,
    pub source_lock: NixpkgsSourceLock,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogBlocker {
    pub code: String,
    pub subject: String,
    pub message: String,
}

impl CatalogBlocker {
    pub fn new(code: &str, subject: &str, message: &str) -> Self {
        assert!(!code.is_empty(), "blocker code must not be empty");
        assert!(!subject.is_empty(), "blocker subject must not be empty");
        Self {
            code: code.into(),
            subject: subject.into(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ForeignNodeFact {
    pub foreign_identity: String,
    pub node_id: String,
    pub canonical_digest_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRequirementFact {
    pub identity: String,
    pub descriptor_digest_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PackageGraphObservation {
    pub selector_name: String,
    pub system: String,
    pub root_node_id: String,
    pub producer_graph_digest_blake3: String,
    pub observed_graph_digest_blake3: String,
    pub producer_index_digest_blake3: String,
    pub observed_index_digest_blake3: String,
    pub nodes: Vec<ForeignNodeFact>,
    pub source_requirements: Vec<SourceRequirementFact>,
    pub blockers: Vec<CatalogBlocker>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MergedNodeFact {
    pub foreign_identity: String,
    pub node_id: String,
    pub canonical_digest_blake3: String,
    pub selectors: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MergedSourceRequirement {
    pub identity: String,
    pub descriptor_digest_blake3: String,
    pub selectors: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum PackageDisposition {
    Buildable {
        root_node_id: String,
        graph_digest_blake3: String,
        index_digest_blake3: String,
    },
    Blocked {
        blockers: Vec<CatalogBlocker>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedPackage {
    pub name: String,
    pub attribute: String,
    pub system: String,
    pub aliases: Vec<String>,
    pub disposition: PackageDisposition,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogPlan {
    pub schema: String,
    pub plan_identity_blake3: String,
    pub manifest: MantlepkgsManifest,
    pub manifest_digest_blake3: String,
    pub producer: ProducerObservation,
    pub packages: Vec<PlannedPackage>,
    pub shared_nodes: Vec<MergedNodeFact>,
    pub source_requirements: Vec<MergedSourceRequirement>,
    pub batch_complete: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactBinding {
    pub role: String,
    pub path: String,
    pub digest_blake3: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProducerSelectionReceipt {
    pub name: String,
    pub attribute: String,
    pub system: String,
    pub root_derivation: String,
    pub graph_digest_blake3: String,
    pub index_digest_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProducerReceipt {
    pub schema: String,
    pub command_class: String,
    pub implementation_id: String,
    pub version: String,
    pub executable_digest_blake3: String,
    pub source_lock: NixpkgsSourceLock,
    pub systems: Vec<String>,
    pub selections: Vec<ProducerSelectionReceipt>,
    pub artifacts: Vec<ArtifactBinding>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRequirementInventory {
    pub schema: String,
    pub requirements: Vec<MergedSourceRequirement>,
    pub optional_transports: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MantlepkgsCatalog {
    pub schema: String,
    pub catalog_identity_blake3: String,
    pub plan_identity_blake3: String,
    pub manifest_digest_blake3: String,
    pub source_lock: NixpkgsSourceLock,
    pub systems: Vec<String>,
    pub target_store_prefix: String,
    pub packages: Vec<PlannedPackage>,
    pub artifacts: Vec<ArtifactBinding>,
    pub producer_receipt_digest_blake3: String,
    pub shared_node_count: u32,
    pub source_requirement_count: u32,
    pub max_artifact_bytes: u64,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactObservation {
    pub path: String,
    pub digest_blake3: String,
    pub bytes: u64,
}

#[derive(Clone)]
struct MergedNodeState {
    node: ForeignNodeFact,
    owners: BTreeSet<PackageKey>,
}

#[derive(Clone)]
struct MergedSourceState {
    source: SourceRequirementFact,
    owners: BTreeSet<PackageKey>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PackageKey {
    system: String,
    name: String,
}

pub fn plan_catalog(
    manifest: &MantlepkgsManifest,
    producer: &ProducerObservation,
    observations: &[PackageGraphObservation],
) -> Result<CatalogPlan, CoreFailure> {
    let normalized = normalize_manifest(manifest)?;
    let manifest_digest = manifest_digest_blake3(&normalized)?;
    let mut diagnostics = validate_producer(&normalized, producer);
    let observations_by_key = collect_observations(observations, &mut diagnostics);
    let mut blockers = normalized
        .selectors
        .iter()
        .map(|selector| (selector_key(selector), Vec::new()))
        .collect::<BTreeMap<_, _>>();
    for diagnostic in &diagnostics {
        for package_blockers in blockers.values_mut() {
            package_blockers.push(CatalogBlocker::new(&diagnostic.code, &diagnostic.path, &diagnostic.message));
        }
    }
    let mut nodes = BTreeMap::<String, MergedNodeState>::new();
    let mut sources = BTreeMap::<String, MergedSourceState>::new();
    for selector in &normalized.selectors {
        let key = selector_key(selector);
        match observations_by_key.get(&key) {
            Some(observation) => {
                merge_observation(&normalized, selector, observation, &mut blockers, &mut nodes, &mut sources)
            }
            None => blockers.entry(key).or_default().push(CatalogBlocker::new(
                "missing-producer-selection",
                &selector.name,
                "the producer did not return facts for this selector",
            )),
        }
    }
    reject_extra_observations(&normalized, &observations_by_key, &mut diagnostics);
    let packages = planned_packages(&normalized.selectors, &observations_by_key, &mut blockers);
    let shared_nodes = merged_nodes(nodes);
    let source_requirements = merged_sources(sources);
    validate_merged_limits(&normalized, &shared_nodes, &source_requirements, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    let is_batch_complete = diagnostics.is_empty()
        && packages.iter().all(|package| matches!(package.disposition, PackageDisposition::Buildable { .. }));
    let non_claims = mantlepkgs_non_claims();
    let plan_identity = plan_identity(
        &normalized,
        &manifest_digest,
        producer,
        &packages,
        &shared_nodes,
        &source_requirements,
        is_batch_complete,
        &diagnostics,
        &non_claims,
    )?;
    debug_assert_eq!(packages.len(), normalized.selectors.len());
    debug_assert_eq!(is_batch_complete, diagnostics.is_empty() && packages.iter().all(is_buildable));
    Ok(CatalogPlan {
        schema: CATALOG_PLAN_SCHEMA.into(),
        plan_identity_blake3: plan_identity,
        manifest: normalized,
        manifest_digest_blake3: manifest_digest,
        producer: producer.clone(),
        packages,
        shared_nodes,
        source_requirements,
        batch_complete: is_batch_complete,
        diagnostics,
        non_claims,
    })
}

pub fn source_requirement_inventory(plan: &CatalogPlan) -> SourceRequirementInventory {
    assert_eq!(plan.schema, CATALOG_PLAN_SCHEMA);
    SourceRequirementInventory {
        schema: SOURCE_INVENTORY_SCHEMA.into(),
        requirements: plan.source_requirements.clone(),
        optional_transports: plan.manifest.source_policy.optional_transports.clone(),
        non_claims: alloc::vec![
            "source-inventory-is-not-source-availability".into(),
            "transport-identity-is-not-recipe-identity".into(),
        ],
    }
}

pub fn build_producer_receipt(
    plan: &CatalogPlan,
    selections: &[ProducerSelectionReceipt],
    artifacts: &[ArtifactBinding],
) -> Result<ProducerReceipt, CoreFailure> {
    if !plan.batch_complete {
        return Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "incomplete-batch",
            "plan.batch_complete",
            "a failed package batch cannot emit a producer receipt",
        )));
    }
    let mut normalized_selections = selections.to_vec();
    normalized_selections.sort_by(|left, right| {
        (&left.system, &left.name, &left.attribute).cmp(&(&right.system, &right.name, &right.attribute))
    });
    validate_selection_receipts(plan, &normalized_selections)?;
    let normalized_artifacts = normalize_artifacts(artifacts, required_producer_roles())?;
    debug_assert_eq!(normalized_selections.len(), plan.packages.len());
    debug_assert_eq!(normalized_artifacts.len(), REQUIRED_ARTIFACT_ROLE_COUNT.saturating_sub(1));
    Ok(ProducerReceipt {
        schema: PRODUCER_RECEIPT_SCHEMA.into(),
        command_class: plan.producer.command_class.clone(),
        implementation_id: plan.producer.implementation_id.clone(),
        version: plan.producer.version.clone(),
        executable_digest_blake3: plan.producer.executable_digest_blake3.clone(),
        source_lock: plan.producer.source_lock.clone(),
        systems: plan.manifest.systems.clone(),
        selections: normalized_selections,
        artifacts: normalized_artifacts,
        non_claims: producer_non_claims(),
    })
}

pub fn producer_receipt_digest_blake3(receipt: &ProducerReceipt) -> Result<String, CoreFailure> {
    if receipt.schema != PRODUCER_RECEIPT_SCHEMA {
        return Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "unsupported-producer-receipt-schema",
            "producer_receipt.schema",
            "the producer receipt schema is not supported",
        )));
    }
    digest_serializable(PRODUCER_RECEIPT_DOMAIN, receipt, "producer-receipt-serialization-failed")
}

pub fn finalize_catalog(
    plan: &CatalogPlan,
    producer_receipt_digest: &str,
    artifacts: &[ArtifactBinding],
) -> Result<MantlepkgsCatalog, CoreFailure> {
    if !plan.batch_complete {
        return Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "incomplete-batch",
            "plan.batch_complete",
            "a failed package batch cannot publish a success catalog",
        )));
    }
    require_digest(producer_receipt_digest, "producer_receipt_digest_blake3")?;
    let normalized_artifacts = normalize_artifacts(artifacts, required_catalog_roles())?;
    let shared_node_count = u32::try_from(plan.shared_nodes.len()).map_err(|_| {
        CoreFailure::from_diagnostic(Diagnostic::new(
            "shared-node-count-overflow",
            "plan.shared_nodes",
            "the shared node count does not fit the catalog field",
        ))
    })?;
    let source_requirement_count = u32::try_from(plan.source_requirements.len()).map_err(|_| {
        CoreFailure::from_diagnostic(Diagnostic::new(
            "source-requirement-count-overflow",
            "plan.source_requirements",
            "the source requirement count does not fit the catalog field",
        ))
    })?;
    let identity = catalog_identity(
        plan,
        producer_receipt_digest,
        &normalized_artifacts,
        shared_node_count,
        source_requirement_count,
    )?;
    debug_assert_eq!(usize::try_from(shared_node_count), Ok(plan.shared_nodes.len()));
    debug_assert_eq!(normalized_artifacts.len(), REQUIRED_ARTIFACT_ROLE_COUNT);
    Ok(MantlepkgsCatalog {
        schema: CATALOG_SCHEMA.into(),
        catalog_identity_blake3: identity,
        plan_identity_blake3: plan.plan_identity_blake3.clone(),
        manifest_digest_blake3: plan.manifest_digest_blake3.clone(),
        source_lock: plan.manifest.source.clone(),
        systems: plan.manifest.systems.clone(),
        target_store_prefix: plan.manifest.conversion_policy.target_store_prefix.clone(),
        packages: plan.packages.clone(),
        artifacts: normalized_artifacts,
        producer_receipt_digest_blake3: producer_receipt_digest.into(),
        shared_node_count,
        source_requirement_count,
        max_artifact_bytes: plan.manifest.limits.max_artifact_bytes,
        non_claims: plan.non_claims.clone(),
    })
}

pub fn validate_catalog_identity(catalog: &MantlepkgsCatalog) -> Result<(), CoreFailure> {
    if catalog.schema != CATALOG_SCHEMA {
        return Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "unsupported-catalog-schema",
            "catalog.schema",
            "the catalog schema is not supported",
        )));
    }
    require_digest(&catalog.plan_identity_blake3, "catalog.plan_identity_blake3")?;
    require_digest(&catalog.manifest_digest_blake3, "catalog.manifest_digest_blake3")?;
    require_digest(&catalog.producer_receipt_digest_blake3, "catalog.producer_receipt_digest_blake3")?;
    debug_assert_eq!(catalog.schema, CATALOG_SCHEMA);
    debug_assert!(is_digest(&catalog.producer_receipt_digest_blake3));
    let observed = standalone_catalog_identity(catalog)?;
    if observed != catalog.catalog_identity_blake3 {
        return Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "catalog-identity-mismatch",
            "catalog.catalog_identity_blake3",
            "the catalog identity does not match its canonical fields",
        )));
    }
    Ok(())
}

pub fn validate_catalog_artifacts(
    catalog: &MantlepkgsCatalog,
    observations: &[ArtifactObservation],
) -> Result<(), CoreFailure> {
    validate_catalog_identity(catalog)?;
    let observed = observations.iter().map(|item| (item.path.as_str(), item)).collect::<BTreeMap<_, _>>();
    let diagnostic_capacity_items = catalog.artifacts.len().saturating_add(1);
    let mut diagnostics = Vec::with_capacity(diagnostic_capacity_items);
    for artifact in &catalog.artifacts {
        validate_relative_artifact_path(&artifact.path, &artifact.role, &mut diagnostics);
        match observed.get(artifact.path.as_str()) {
            Some(item) if item.digest_blake3 == artifact.digest_blake3 && item.bytes == artifact.bytes => {}
            Some(_) => diagnostics.push(Diagnostic::new(
                "catalog-artifact-mismatch",
                &artifact.path,
                "the observed artifact bytes or digest do not match the catalog",
            )),
            None => diagnostics.push(Diagnostic::new(
                "catalog-artifact-missing",
                &artifact.path,
                "the catalog artifact is missing",
            )),
        }
    }
    if observations.len() != catalog.artifacts.len() {
        diagnostics.push(Diagnostic::new(
            "catalog-artifact-set-mismatch",
            "catalog.artifacts",
            "the observed artifact set differs from the catalog artifact set",
        ));
    }
    if diagnostics.is_empty() {
        debug_assert_eq!(observed.len(), catalog.artifacts.len());
        debug_assert_eq!(observations.len(), catalog.artifacts.len());
        Ok(())
    } else {
        Err(CoreFailure::from_diagnostics(diagnostics))
    }
}

#[allow(tigerstyle::ambiguous_params)] // Package name and system are distinct lookup dimensions checked by filters below.
pub fn lookup_catalog_package<'a>(
    catalog: &'a MantlepkgsCatalog,
    name: &str,
    system: &str,
) -> Result<&'a PlannedPackage, CoreFailure> {
    let matches = catalog
        .packages
        .iter()
        .filter(|package| package.system == system)
        .filter(|package| package.name == name || package.aliases.iter().any(|alias| alias == name))
        .collect::<Vec<_>>();
    debug_assert!(matches.len() <= catalog.packages.len());
    debug_assert!(matches.iter().all(|package| package.system == system));
    match matches.as_slice() {
        [package] if matches!(package.disposition, PackageDisposition::Buildable { .. }) => Ok(package),
        [package] => Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "catalog-package-blocked",
            &package.name,
            "the selected package has blockers and is not buildable",
        ))),
        [] => Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "catalog-package-missing",
            name,
            "the selected package is absent for the requested system",
        ))),
        _ => Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "catalog-package-ambiguous",
            name,
            "the selected package or alias is ambiguous",
        ))),
    }
}

fn validate_producer(manifest: &MantlepkgsManifest, producer: &ProducerObservation) -> Vec<Diagnostic> {
    const PRODUCER_FIELD_COUNT: usize = 5;
    let mut diagnostics = Vec::with_capacity(PRODUCER_FIELD_COUNT);
    if producer.command_class != PRODUCER_COMMAND_CLASS {
        diagnostics.push(Diagnostic::new(
            "unsupported-producer-command-class",
            "producer.command_class",
            "the producer command class is not supported",
        ));
    }
    if producer.implementation_id.is_empty() || producer.implementation_id.len() > MAX_IMPLEMENTATION_ID_BYTES {
        diagnostics.push(Diagnostic::new(
            "invalid-producer-implementation",
            "producer.implementation_id",
            "the producer implementation identity is empty or too large",
        ));
    }
    if producer.version.is_empty() || producer.version.len() > MAX_PRODUCER_VERSION_BYTES {
        diagnostics.push(Diagnostic::new(
            "invalid-producer-version",
            "producer.version",
            "the producer version is empty or too large",
        ));
    }
    if !is_digest(&producer.executable_digest_blake3) {
        diagnostics.push(Diagnostic::new(
            "invalid-producer-executable-digest",
            "producer.executable_digest_blake3",
            "the producer executable digest is not a BLAKE3 hexadecimal value",
        ));
    }
    if producer.source_lock != manifest.source {
        diagnostics.push(Diagnostic::new(
            "producer-source-lock-mismatch",
            "producer.source_lock",
            "the producer source lock differs from the manifest source lock",
        ));
    }
    debug_assert!(diagnostics.len() <= PRODUCER_FIELD_COUNT);
    debug_assert!(diagnostics.iter().all(|diagnostic| !diagnostic.code.is_empty()));
    diagnostics
}

fn collect_observations(
    observations: &[PackageGraphObservation],
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeMap<PackageKey, PackageGraphObservation> {
    let mut collected = BTreeMap::new();
    for observation in observations {
        let key = PackageKey {
            system: observation.system.clone(),
            name: observation.selector_name.clone(),
        };
        if collected.insert(key.clone(), observation.clone()).is_some() {
            diagnostics.push(Diagnostic::new(
                "duplicate-producer-selection",
                &format!("{}:{}", key.system, key.name),
                "the producer returned more than one observation for a selector",
            ));
        }
    }
    collected
}

#[allow(tigerstyle::too_many_parameters)] // The pure merge threads one manifest, one observation, and three canonical output maps.
fn merge_observation(
    manifest: &MantlepkgsManifest,
    selector: &PackageSelector,
    observation: &PackageGraphObservation,
    blockers: &mut BTreeMap<PackageKey, Vec<CatalogBlocker>>,
    nodes: &mut BTreeMap<String, MergedNodeState>,
    sources: &mut BTreeMap<String, MergedSourceState>,
) {
    let key = selector_key(selector);
    let package_blockers = blockers.entry(key.clone()).or_default();
    package_blockers.extend(observation.blockers.clone());
    compare_digest_pair(
        &observation.producer_graph_digest_blake3,
        &observation.observed_graph_digest_blake3,
        "stale-graph-digest",
        &selector.name,
        package_blockers,
    );
    compare_digest_pair(
        &observation.producer_index_digest_blake3,
        &observation.observed_index_digest_blake3,
        "stale-index-digest",
        &selector.name,
        package_blockers,
    );
    if observation.nodes.is_empty() {
        package_blockers.push(CatalogBlocker::new(
            "empty-package-graph",
            &selector.name,
            "the package graph has no nodes",
        ));
    }
    if exceeds_u32_limit(observation.nodes.len(), manifest.limits.max_graph_nodes) {
        package_blockers.push(CatalogBlocker::new(
            "package-graph-node-limit-exceeded",
            &selector.name,
            "the package graph exceeds the named node limit",
        ));
    }
    if !observation.nodes.iter().any(|node| node.node_id == observation.root_node_id) {
        package_blockers.push(CatalogBlocker::new(
            "missing-package-root",
            &selector.name,
            "the selected package root is absent from its graph",
        ));
    }
    merge_nodes(&key, &observation.nodes, blockers, nodes);
    merge_sources(&key, &observation.source_requirements, blockers, sources);
    debug_assert!(blockers.contains_key(&key));
    debug_assert!(nodes.values().all(|state| !state.owners.is_empty()));
}

fn merge_nodes(
    owner: &PackageKey,
    facts: &[ForeignNodeFact],
    blockers: &mut BTreeMap<PackageKey, Vec<CatalogBlocker>>,
    merged: &mut BTreeMap<String, MergedNodeState>,
) {
    let mut local = BTreeSet::new();
    for fact in facts {
        if !local.insert(fact.foreign_identity.as_str()) {
            add_blocker(
                blockers,
                owner,
                CatalogBlocker::new(
                    "duplicate-foreign-node",
                    &fact.foreign_identity,
                    "the package graph repeats a foreign node identity",
                ),
            );
            continue;
        }
        if !is_digest(&fact.canonical_digest_blake3) {
            add_blocker(
                blockers,
                owner,
                CatalogBlocker::new(
                    "invalid-node-digest",
                    &fact.foreign_identity,
                    "the foreign node canonical digest is invalid",
                ),
            );
            continue;
        }
        match merged.get_mut(&fact.foreign_identity) {
            Some(state)
                if state.node.node_id != fact.node_id
                    || state.node.canonical_digest_blake3 != fact.canonical_digest_blake3 =>
            {
                let conflict = CatalogBlocker::new(
                    "conflicting-foreign-node",
                    &fact.foreign_identity,
                    "the same foreign identity has conflicting canonical facts",
                );
                for previous in &state.owners {
                    add_blocker(blockers, previous, conflict.clone());
                }
                add_blocker(blockers, owner, conflict);
                state.owners.insert(owner.clone());
            }
            Some(state) => {
                state.owners.insert(owner.clone());
            }
            None => {
                merged.insert(fact.foreign_identity.clone(), MergedNodeState {
                    node: fact.clone(),
                    owners: BTreeSet::from([owner.clone()]),
                });
            }
        }
    }
    debug_assert!(blockers.contains_key(owner));
    debug_assert!(merged.values().all(|state| !state.owners.is_empty()));
}

fn merge_sources(
    owner: &PackageKey,
    facts: &[SourceRequirementFact],
    blockers: &mut BTreeMap<PackageKey, Vec<CatalogBlocker>>,
    merged: &mut BTreeMap<String, MergedSourceState>,
) {
    let mut local = BTreeSet::new();
    for fact in facts {
        if !local.insert(fact.identity.as_str()) {
            add_blocker(
                blockers,
                owner,
                CatalogBlocker::new(
                    "duplicate-source-requirement",
                    &fact.identity,
                    "the package graph repeats a source requirement",
                ),
            );
            continue;
        }
        if !is_digest(&fact.descriptor_digest_blake3) {
            add_blocker(
                blockers,
                owner,
                CatalogBlocker::new(
                    "invalid-source-requirement-digest",
                    &fact.identity,
                    "the source requirement descriptor digest is invalid",
                ),
            );
            continue;
        }
        match merged.get_mut(&fact.identity) {
            Some(state) if state.source.descriptor_digest_blake3 != fact.descriptor_digest_blake3 => {
                let conflict = CatalogBlocker::new(
                    "conflicting-source-requirement",
                    &fact.identity,
                    "the same source identity has conflicting descriptor facts",
                );
                for previous in &state.owners {
                    add_blocker(blockers, previous, conflict.clone());
                }
                add_blocker(blockers, owner, conflict);
                state.owners.insert(owner.clone());
            }
            Some(state) => {
                state.owners.insert(owner.clone());
            }
            None => {
                merged.insert(fact.identity.clone(), MergedSourceState {
                    source: fact.clone(),
                    owners: BTreeSet::from([owner.clone()]),
                });
            }
        }
    }
    debug_assert!(blockers.contains_key(owner));
    debug_assert!(merged.values().all(|state| !state.owners.is_empty()));
}

fn planned_packages(
    selectors: &[PackageSelector],
    observations: &BTreeMap<PackageKey, PackageGraphObservation>,
    blockers: &mut BTreeMap<PackageKey, Vec<CatalogBlocker>>,
) -> Vec<PlannedPackage> {
    selectors
        .iter()
        .map(|selector| {
            let key = selector_key(selector);
            let mut package_blockers = blockers.remove(&key).unwrap_or_default();
            package_blockers.sort();
            package_blockers.dedup();
            if package_blockers.len() > MAX_BLOCKERS_PER_PACKAGE {
                package_blockers.truncate(MAX_BLOCKERS_PER_PACKAGE);
                package_blockers.push(CatalogBlocker::new(
                    "blocker-limit-exceeded",
                    &selector.name,
                    "the package blocker list exceeded the reporting limit",
                ));
            }
            let disposition = match (package_blockers.is_empty(), observations.get(&key)) {
                (true, Some(observation)) => PackageDisposition::Buildable {
                    root_node_id: observation.root_node_id.clone(),
                    graph_digest_blake3: observation.observed_graph_digest_blake3.clone(),
                    index_digest_blake3: observation.observed_index_digest_blake3.clone(),
                },
                _ => PackageDisposition::Blocked {
                    blockers: package_blockers,
                },
            };
            PlannedPackage {
                name: selector.name.clone(),
                attribute: selector.attribute.clone(),
                system: selector.system.clone(),
                aliases: selector.aliases.clone(),
                disposition,
            }
        })
        .collect()
}

fn merged_nodes(nodes: BTreeMap<String, MergedNodeState>) -> Vec<MergedNodeFact> {
    nodes
        .into_values()
        .map(|state| MergedNodeFact {
            foreign_identity: state.node.foreign_identity,
            node_id: state.node.node_id,
            canonical_digest_blake3: state.node.canonical_digest_blake3,
            selectors: state.owners.into_iter().map(|owner| owner.name).collect(),
        })
        .collect()
}

fn merged_sources(sources: BTreeMap<String, MergedSourceState>) -> Vec<MergedSourceRequirement> {
    sources
        .into_values()
        .map(|state| MergedSourceRequirement {
            identity: state.source.identity,
            descriptor_digest_blake3: state.source.descriptor_digest_blake3,
            selectors: state.owners.into_iter().map(|owner| owner.name).collect(),
        })
        .collect()
}

fn validate_merged_limits(
    manifest: &MantlepkgsManifest,
    nodes: &[MergedNodeFact],
    sources: &[MergedSourceRequirement],
    diagnostics: &mut Vec<Diagnostic>,
) {
    if exceeds_u32_limit(nodes.len(), manifest.limits.max_graph_nodes) {
        diagnostics.push(Diagnostic::new(
            "catalog-graph-node-limit-exceeded",
            "shared_nodes",
            "the merged graph exceeds the named node limit",
        ));
    }
    if exceeds_u32_limit(sources.len(), manifest.limits.max_source_requirements) {
        diagnostics.push(Diagnostic::new(
            "source-requirement-limit-exceeded",
            "source_requirements",
            "the merged source inventory exceeds the named limit",
        ));
    }
}

fn normalize_artifacts(
    artifacts: &[ArtifactBinding],
    required_roles: BTreeSet<&'static str>,
) -> Result<Vec<ArtifactBinding>, CoreFailure> {
    let mut normalized = artifacts.to_vec();
    normalized.sort();
    let diagnostic_capacity_items = normalized.len().saturating_add(1);
    let mut diagnostics = Vec::with_capacity(diagnostic_capacity_items);
    let mut roles = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for artifact in &normalized {
        validate_relative_artifact_path(&artifact.path, &artifact.role, &mut diagnostics);
        if !roles.insert(artifact.role.as_str()) {
            diagnostics.push(Diagnostic::new(
                "duplicate-artifact-role",
                &artifact.role,
                "the artifact role is repeated",
            ));
        }
        if !paths.insert(artifact.path.as_str()) {
            diagnostics.push(Diagnostic::new(
                "duplicate-artifact-path",
                &artifact.path,
                "the artifact path is repeated",
            ));
        }
        if !is_digest(&artifact.digest_blake3) {
            diagnostics.push(Diagnostic::new(
                "invalid-artifact-digest",
                &artifact.path,
                "the artifact digest is not a BLAKE3 hexadecimal value",
            ));
        }
        if artifact.bytes == 0 {
            diagnostics.push(Diagnostic::new(
                "empty-artifact",
                &artifact.path,
                "the artifact byte count must be positive",
            ));
        }
    }
    let observed_roles = normalized.iter().map(|artifact| artifact.role.as_str()).collect::<BTreeSet<_>>();
    if observed_roles != required_roles {
        diagnostics.push(Diagnostic::new(
            "artifact-role-set-mismatch",
            "artifacts",
            "the artifact roles differ from the required publication set",
        ));
    }
    if diagnostics.is_empty() {
        debug_assert_eq!(roles.len(), normalized.len());
        debug_assert_eq!(paths.len(), normalized.len());
        Ok(normalized)
    } else {
        Err(CoreFailure::from_diagnostics(diagnostics))
    }
}

fn validate_selection_receipts(plan: &CatalogPlan, selections: &[ProducerSelectionReceipt]) -> Result<(), CoreFailure> {
    let planned = plan
        .packages
        .iter()
        .map(|package| (package.system.as_str(), package.name.as_str(), package.attribute.as_str()))
        .collect::<BTreeSet<_>>();
    let observed = selections
        .iter()
        .map(|selection| (selection.system.as_str(), selection.name.as_str(), selection.attribute.as_str()))
        .collect::<BTreeSet<_>>();
    let diagnostic_capacity_items = selections.len().saturating_add(1);
    let mut diagnostics = Vec::with_capacity(diagnostic_capacity_items);
    if planned != observed || observed.len() != selections.len() {
        diagnostics.push(Diagnostic::new(
            "producer-selection-set-mismatch",
            "producer_receipt.selections",
            "the producer receipt selection set differs from the completed plan",
        ));
    }
    for selection in selections {
        if selection.root_derivation.is_empty() {
            diagnostics.push(Diagnostic::new(
                "missing-root-derivation",
                &selection.name,
                "the producer selection has no root derivation",
            ));
        }
        if !is_digest(&selection.graph_digest_blake3) || !is_digest(&selection.index_digest_blake3) {
            diagnostics.push(Diagnostic::new(
                "invalid-selection-artifact-digest",
                &selection.name,
                "the selection graph or index digest is invalid",
            ));
        }
    }
    if diagnostics.is_empty() {
        debug_assert_eq!(planned.len(), plan.packages.len());
        debug_assert_eq!(observed.len(), selections.len());
        Ok(())
    } else {
        Err(CoreFailure::from_diagnostics(diagnostics))
    }
}

// Canonical preimage fields stay explicit to prevent accidental identity omissions.
#[allow(clippy::too_many_arguments)]
#[allow(tigerstyle::too_many_parameters)] // Canonical preimage fields map one-to-one into the local typed struct.
fn plan_identity(
    manifest: &MantlepkgsManifest,
    manifest_digest: &str,
    producer: &ProducerObservation,
    packages: &[PlannedPackage],
    nodes: &[MergedNodeFact],
    sources: &[MergedSourceRequirement],
    batch_complete: bool,
    diagnostics: &[Diagnostic],
    non_claims: &[String],
) -> Result<String, CoreFailure> {
    #[derive(Serialize)]
    struct Preimage<'a> {
        schema: &'a str,
        manifest: &'a MantlepkgsManifest,
        manifest_digest_blake3: &'a str,
        producer: &'a ProducerObservation,
        packages: &'a [PlannedPackage],
        shared_nodes: &'a [MergedNodeFact],
        source_requirements: &'a [MergedSourceRequirement],
        batch_complete: bool,
        diagnostics: &'a [Diagnostic],
        non_claims: &'a [String],
    }
    debug_assert_eq!(manifest.schema, crate::MANIFEST_SCHEMA);
    debug_assert!(is_digest(manifest_digest));
    digest_serializable(
        PLAN_DOMAIN,
        &Preimage {
            schema: CATALOG_PLAN_SCHEMA,
            manifest,
            manifest_digest_blake3: manifest_digest,
            producer,
            packages,
            shared_nodes: nodes,
            source_requirements: sources,
            batch_complete,
            diagnostics,
            non_claims,
        },
        "catalog-plan-serialization-failed",
    )
}

#[allow(tigerstyle::ambiguous_params)] // Shared-node and source counts name distinct catalog fields checked against the plan.
fn catalog_identity(
    plan: &CatalogPlan,
    producer_receipt_digest: &str,
    artifacts: &[ArtifactBinding],
    shared_node_count: u32,
    source_requirement_count: u32,
) -> Result<String, CoreFailure> {
    #[derive(Serialize)]
    struct Preimage<'a> {
        schema: &'a str,
        plan_identity_blake3: &'a str,
        manifest_digest_blake3: &'a str,
        source_lock: &'a NixpkgsSourceLock,
        systems: &'a [String],
        target_store_prefix: &'a str,
        packages: &'a [PlannedPackage],
        artifacts: &'a [ArtifactBinding],
        producer_receipt_digest_blake3: &'a str,
        shared_node_count: u32,
        source_requirement_count: u32,
        max_artifact_bytes: u64,
        non_claims: &'a [String],
    }
    debug_assert_eq!(usize::try_from(shared_node_count), Ok(plan.shared_nodes.len()));
    debug_assert_eq!(usize::try_from(source_requirement_count), Ok(plan.source_requirements.len()));
    digest_serializable(
        CATALOG_DOMAIN,
        &Preimage {
            schema: CATALOG_SCHEMA,
            plan_identity_blake3: &plan.plan_identity_blake3,
            manifest_digest_blake3: &plan.manifest_digest_blake3,
            source_lock: &plan.manifest.source,
            systems: &plan.manifest.systems,
            target_store_prefix: &plan.manifest.conversion_policy.target_store_prefix,
            packages: &plan.packages,
            artifacts,
            producer_receipt_digest_blake3: producer_receipt_digest,
            shared_node_count,
            source_requirement_count,
            max_artifact_bytes: plan.manifest.limits.max_artifact_bytes,
            non_claims: &plan.non_claims,
        },
        "catalog-serialization-failed",
    )
}

fn standalone_catalog_identity(catalog: &MantlepkgsCatalog) -> Result<String, CoreFailure> {
    debug_assert_eq!(catalog.schema, CATALOG_SCHEMA);
    debug_assert!(is_digest(&catalog.producer_receipt_digest_blake3));
    #[derive(Serialize)]
    struct Preimage<'a> {
        schema: &'a str,
        plan_identity_blake3: &'a str,
        manifest_digest_blake3: &'a str,
        source_lock: &'a NixpkgsSourceLock,
        systems: &'a [String],
        target_store_prefix: &'a str,
        packages: &'a [PlannedPackage],
        artifacts: &'a [ArtifactBinding],
        producer_receipt_digest_blake3: &'a str,
        shared_node_count: u32,
        source_requirement_count: u32,
        max_artifact_bytes: u64,
        non_claims: &'a [String],
    }
    digest_serializable(
        CATALOG_DOMAIN,
        &Preimage {
            schema: &catalog.schema,
            plan_identity_blake3: &catalog.plan_identity_blake3,
            manifest_digest_blake3: &catalog.manifest_digest_blake3,
            source_lock: &catalog.source_lock,
            systems: &catalog.systems,
            target_store_prefix: &catalog.target_store_prefix,
            packages: &catalog.packages,
            artifacts: &catalog.artifacts,
            producer_receipt_digest_blake3: &catalog.producer_receipt_digest_blake3,
            shared_node_count: catalog.shared_node_count,
            source_requirement_count: catalog.source_requirement_count,
            max_artifact_bytes: catalog.max_artifact_bytes,
            non_claims: &catalog.non_claims,
        },
        "catalog-serialization-failed",
    )
}

fn digest_serializable<T: Serialize>(domain: &[u8], value: &T, code: &str) -> Result<String, CoreFailure> {
    let bytes = serde_json::to_vec(value).map_err(|_| {
        CoreFailure::from_diagnostic(Diagnostic::new(
            code,
            "identity",
            "the canonical identity preimage did not serialize",
        ))
    })?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(&bytes);
    Ok(hasher.finalize().to_hex().as_str().into())
}

#[allow(tigerstyle::ambiguous_params)] // Expected, observed, diagnostic code, and subject are separate comparison roles.
fn compare_digest_pair(expected: &str, observed: &str, code: &str, subject: &str, blockers: &mut Vec<CatalogBlocker>) {
    if !is_digest(expected) || !is_digest(observed) || expected != observed {
        blockers.push(CatalogBlocker::new(
            code,
            subject,
            "the producer digest does not match the observed artifact digest",
        ));
    }
}

fn reject_extra_observations(
    manifest: &MantlepkgsManifest,
    observations: &BTreeMap<PackageKey, PackageGraphObservation>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let selectors = manifest.selectors.iter().map(selector_key).collect::<BTreeSet<_>>();
    for key in observations.keys() {
        if !selectors.contains(key) {
            diagnostics.push(Diagnostic::new(
                "unexpected-producer-selection",
                &format!("{}:{}", key.system, key.name),
                "the producer returned a package that the manifest did not select",
            ));
        }
    }
}

fn add_blocker(blockers: &mut BTreeMap<PackageKey, Vec<CatalogBlocker>>, owner: &PackageKey, blocker: CatalogBlocker) {
    blockers.entry(owner.clone()).or_default().push(blocker);
}

fn selector_key(selector: &PackageSelector) -> PackageKey {
    PackageKey {
        system: selector.system.clone(),
        name: selector.name.clone(),
    }
}

fn is_buildable(package: &PlannedPackage) -> bool {
    matches!(package.disposition, PackageDisposition::Buildable { .. })
}

#[allow(tigerstyle::ambiguous_params)] // Artifact path and role map directly to separate diagnostic fields.
fn validate_relative_artifact_path(path: &str, role: &str, diagnostics: &mut Vec<Diagnostic>) {
    let is_unsafe_path = path.is_empty()
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains('\\')
        || path.split('/').any(|component| component.is_empty() || component == "." || component == "..");
    if is_unsafe_path {
        diagnostics.push(Diagnostic::new(
            "unsafe-artifact-path",
            role,
            "the artifact path must be normalized, relative, and confined",
        ));
    }
}

#[allow(tigerstyle::ambiguous_params)] // Digest value and field path are distinct validation inputs.
fn require_digest(value: &str, path: &str) -> Result<(), CoreFailure> {
    if is_digest(value) {
        Ok(())
    } else {
        Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "invalid-blake3-digest",
            path,
            "the digest is not a lowercase BLAKE3 hexadecimal value",
        )))
    }
}

fn is_digest(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn exceeds_u32_limit(observed_items: usize, limit_items: u32) -> bool {
    match u32::try_from(observed_items) {
        Ok(observed_items_u32) => observed_items_u32 > limit_items,
        Err(_) => true,
    }
}

fn required_producer_roles() -> BTreeSet<&'static str> {
    BTreeSet::from([
        ROLE_SHARED_GRAPH,
        ROLE_PACKAGE_INDEX,
        ROLE_SOURCE_INVENTORY,
        ROLE_TRANSLATION_POLICY,
        ROLE_EXECUTION_PROFILE,
    ])
}

fn required_catalog_roles() -> BTreeSet<&'static str> {
    let mut roles = required_producer_roles();
    roles.insert(ROLE_PRODUCER_RECEIPT);
    debug_assert_eq!(roles.len(), REQUIRED_ARTIFACT_ROLE_COUNT);
    roles
}

pub fn mantlepkgs_non_claims() -> Vec<String> {
    alloc::vec![
        "not-nix-source-translation".into(),
        "not-nix-evaluator-parity".into(),
        "not-full-nixpkgs-coverage".into(),
        "not-package-correctness".into(),
        "not-reproducibility".into(),
        "not-bootstrap-parity".into(),
        "not-release-eligibility".into(),
    ]
}

fn producer_non_claims() -> Vec<String> {
    alloc::vec![
        "producer-receipt-is-not-build-success".into(),
        "producer-receipt-is-not-package-correctness".into(),
        "producer-receipt-is-not-reproducibility".into(),
        "producer-receipt-is-not-release-eligibility".into(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConversionPolicy;
    use crate::ManifestArtifact;
    use crate::ManifestLimits;
    use crate::OutputLayout;
    use crate::RECOMPUTE_CONVERSION_MODE;
    use crate::SOURCE_BUNDLE_POLICY_MODE;
    use crate::SUPPORTED_SYSTEM_X86_64_LINUX;
    use crate::SourcePolicy;

    const REVISION: &str = "0123456789abcdef0123456789abcdef01234567";
    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const SELECTOR_LIMIT: u32 = 8;
    const NODE_LIMIT: u32 = 1_024;
    const GRAPH_BYTE_LIMIT: u64 = 16_777_216;
    const SOURCE_LIMIT: u32 = 1_024;
    const ARTIFACT_LIMIT: u64 = 33_554_432;
    const ARTIFACT_BYTES: u64 = 64;

    fn manifest() -> MantlepkgsManifest {
        MantlepkgsManifest {
            schema: crate::MANIFEST_SCHEMA.into(),
            source: source_lock(),
            systems: alloc::vec![SUPPORTED_SYSTEM_X86_64_LINUX.into()],
            selectors: alloc::vec![
                PackageSelector {
                    name: "app".into(),
                    attribute: "app".into(),
                    system: SUPPORTED_SYSTEM_X86_64_LINUX.into(),
                    aliases: alloc::vec!["tool".into()],
                },
                PackageSelector {
                    name: "lib".into(),
                    attribute: "lib".into(),
                    system: SUPPORTED_SYSTEM_X86_64_LINUX.into(),
                    aliases: Vec::new(),
                },
            ],
            conversion_policy: ConversionPolicy {
                mode: RECOMPUTE_CONVERSION_MODE.into(),
                target_store_prefix: "/mantle/store".into(),
                translation_policy: ManifestArtifact {
                    path: "policy/translation.json".into(),
                    digest_blake3: DIGEST_A.into(),
                },
                execution_profile: ManifestArtifact {
                    path: "policy/execution.json".into(),
                    digest_blake3: DIGEST_B.into(),
                },
            },
            source_policy: SourcePolicy {
                mode: SOURCE_BUNDLE_POLICY_MODE.into(),
                optional_transports: Vec::new(),
            },
            output: OutputLayout {
                generation_directory: "generations".into(),
            },
            limits: ManifestLimits {
                max_selectors: SELECTOR_LIMIT,
                max_graph_nodes: NODE_LIMIT,
                max_graph_bytes: GRAPH_BYTE_LIMIT,
                max_source_requirements: SOURCE_LIMIT,
                max_artifact_bytes: ARTIFACT_LIMIT,
            },
        }
    }

    fn source_lock() -> NixpkgsSourceLock {
        NixpkgsSourceLock {
            reference: format!("github:NixOS/nixpkgs/{REVISION}"),
            revision: REVISION.into(),
            lock_digest_blake3: DIGEST_C.into(),
        }
    }

    fn producer() -> ProducerObservation {
        ProducerObservation {
            command_class: PRODUCER_COMMAND_CLASS.into(),
            implementation_id: "nix".into(),
            version: "nix (Nix) fixture".into(),
            executable_digest_blake3: DIGEST_A.into(),
            source_lock: source_lock(),
        }
    }

    #[allow(tigerstyle::ambiguous_params)] // Fixture package name and root identity are distinct test data.
    fn observation(name: &str, root: &str, nodes: Vec<ForeignNodeFact>) -> PackageGraphObservation {
        PackageGraphObservation {
            selector_name: name.into(),
            system: SUPPORTED_SYSTEM_X86_64_LINUX.into(),
            root_node_id: root.into(),
            producer_graph_digest_blake3: DIGEST_A.into(),
            observed_graph_digest_blake3: DIGEST_A.into(),
            producer_index_digest_blake3: DIGEST_B.into(),
            observed_index_digest_blake3: DIGEST_B.into(),
            nodes,
            source_requirements: Vec::new(),
            blockers: Vec::new(),
        }
    }

    #[allow(tigerstyle::ambiguous_params)] // Fixture foreign identity, node ID, and digest map to separate fields.
    fn node(identity: &str, node_id: &str, digest: &str) -> ForeignNodeFact {
        ForeignNodeFact {
            foreign_identity: identity.into(),
            node_id: node_id.into(),
            canonical_digest_blake3: digest.into(),
        }
    }

    fn artifacts(include_receipt: bool) -> Vec<ArtifactBinding> {
        let mut artifacts = alloc::vec![
            artifact(ROLE_SHARED_GRAPH, SHARED_GRAPH_PATH, DIGEST_A),
            artifact(ROLE_PACKAGE_INDEX, PACKAGE_INDEX_PATH, DIGEST_B),
            artifact(ROLE_SOURCE_INVENTORY, SOURCE_INVENTORY_PATH, DIGEST_C),
            artifact(ROLE_TRANSLATION_POLICY, TRANSLATION_POLICY_PATH, DIGEST_A),
            artifact(ROLE_EXECUTION_PROFILE, EXECUTION_PROFILE_PATH, DIGEST_B),
        ];
        if include_receipt {
            artifacts.push(artifact(ROLE_PRODUCER_RECEIPT, PRODUCER_RECEIPT_PATH, DIGEST_C));
        }
        artifacts
    }

    #[allow(tigerstyle::ambiguous_params)] // Fixture role, path, and digest map to separate artifact fields.
    fn artifact(role: &str, path: &str, digest: &str) -> ArtifactBinding {
        ArtifactBinding {
            role: role.into(),
            path: path.into(),
            digest_blake3: digest.into(),
            bytes: ARTIFACT_BYTES,
        }
    }

    #[test]
    fn catalog_plan_deduplicates_shared_nodes_and_is_order_independent() {
        let app = observation("app", "app-node", alloc::vec![
            node("/nix/store/shared.drv", "shared", DIGEST_A),
            node("/nix/store/app.drv", "app-node", DIGEST_B)
        ]);
        let library = observation("lib", "shared", alloc::vec![node("/nix/store/shared.drv", "shared", DIGEST_A)]);

        let first = plan_catalog(&manifest(), &producer(), &[app.clone(), library.clone()]).expect("catalog plans");
        let second = plan_catalog(&manifest(), &producer(), &[library, app]).expect("reordered catalog plans");
        assert!(first.batch_complete);
        assert_eq!(first.shared_nodes.len(), 2);
        assert_eq!(first.plan_identity_blake3, second.plan_identity_blake3);
        assert!(first.packages.iter().all(is_buildable));
    }

    #[test]
    fn catalog_plan_blocks_conflicts_stale_digests_and_missing_roots() {
        let app = observation("app", "missing", alloc::vec![node("/nix/store/shared.drv", "shared", DIGEST_A)]);
        let mut library = observation("lib", "other", alloc::vec![node("/nix/store/shared.drv", "other", DIGEST_B)]);
        library.observed_graph_digest_blake3 = DIGEST_C.into();

        let plan = plan_catalog(&manifest(), &producer(), &[app, library]).expect("blocked catalog still plans");
        assert!(!plan.batch_complete);
        assert!(
            plan.packages
                .iter()
                .all(|package| matches!(package.disposition, PackageDisposition::Blocked { .. }))
        );
        let blocker_codes = plan
            .packages
            .iter()
            .flat_map(|package| match &package.disposition {
                PackageDisposition::Blocked { blockers } => blockers.iter().map(|item| item.code.as_str()).collect(),
                PackageDisposition::Buildable { .. } => Vec::new(),
            })
            .collect::<BTreeSet<_>>();
        assert!(blocker_codes.contains("conflicting-foreign-node"));
        assert!(blocker_codes.contains("stale-graph-digest"));
        assert!(blocker_codes.contains("missing-package-root"));
    }

    #[test]
    fn complete_plan_finalizes_and_artifact_tampering_fails() {
        let app = observation("app", "app", alloc::vec![node("/nix/store/app.drv", "app", DIGEST_A)]);
        let library = observation("lib", "lib", alloc::vec![node("/nix/store/lib.drv", "lib", DIGEST_B)]);
        let plan = plan_catalog(&manifest(), &producer(), &[app, library]).expect("catalog plans");
        let selections = alloc::vec![
            ProducerSelectionReceipt {
                name: "app".into(),
                attribute: "app".into(),
                system: SUPPORTED_SYSTEM_X86_64_LINUX.into(),
                root_derivation: "/nix/store/app.drv".into(),
                graph_digest_blake3: DIGEST_A.into(),
                index_digest_blake3: DIGEST_B.into(),
            },
            ProducerSelectionReceipt {
                name: "lib".into(),
                attribute: "lib".into(),
                system: SUPPORTED_SYSTEM_X86_64_LINUX.into(),
                root_derivation: "/nix/store/lib.drv".into(),
                graph_digest_blake3: DIGEST_A.into(),
                index_digest_blake3: DIGEST_B.into(),
            },
        ];
        let receipt = build_producer_receipt(&plan, &selections, &artifacts(false)).expect("receipt builds");
        let receipt_digest = producer_receipt_digest_blake3(&receipt).expect("receipt hashes");
        let catalog = finalize_catalog(&plan, &receipt_digest, &artifacts(true)).expect("catalog finalizes");
        let mut observations = catalog
            .artifacts
            .iter()
            .map(|artifact| ArtifactObservation {
                path: artifact.path.clone(),
                digest_blake3: artifact.digest_blake3.clone(),
                bytes: artifact.bytes,
            })
            .collect::<Vec<_>>();
        validate_catalog_artifacts(&catalog, &observations).expect("matching artifacts validate");
        let mut identity_tamper = catalog.clone();
        identity_tamper.target_store_prefix = "/other/store".into();
        let identity_failure = validate_catalog_identity(&identity_tamper).expect_err("identity tampering fails");
        assert_eq!(identity_failure.diagnostics[0].code, "catalog-identity-mismatch");
        observations[0].digest_blake3 = DIGEST_C.into();
        let failure = validate_catalog_artifacts(&catalog, &observations).expect_err("tampering fails");
        assert!(failure.diagnostics.iter().any(|item| item.code == "catalog-artifact-mismatch"));
    }

    #[test]
    fn stale_producer_lock_blocks_the_complete_selection() {
        let app = observation("app", "app", alloc::vec![node("/nix/store/app.drv", "app", DIGEST_A)]);
        let library = observation("lib", "lib", alloc::vec![node("/nix/store/lib.drv", "lib", DIGEST_B)]);
        let mut stale = producer();
        stale.source_lock.revision = "1111111111111111111111111111111111111111".into();
        stale.source_lock.reference = "github:NixOS/nixpkgs/1111111111111111111111111111111111111111".into();

        let plan = plan_catalog(&manifest(), &stale, &[app, library]).expect("stale lock plans as failure");
        assert!(!plan.batch_complete);
        assert!(plan.diagnostics.iter().any(|item| item.code == "producer-source-lock-mismatch"));
        assert!(
            plan.packages
                .iter()
                .all(|package| matches!(package.disposition, PackageDisposition::Blocked { .. }))
        );
    }

    #[test]
    fn incomplete_plan_cannot_emit_receipt_or_catalog() {
        let plan = plan_catalog(&manifest(), &producer(), &[]).expect("missing observations create blockers");
        assert!(!plan.batch_complete);
        let receipt_failure = build_producer_receipt(&plan, &[], &[]).expect_err("receipt is blocked");
        let catalog_failure = finalize_catalog(&plan, DIGEST_A, &[]).expect_err("catalog is blocked");
        assert_eq!(receipt_failure.diagnostics[0].code, "incomplete-batch");
        assert_eq!(catalog_failure.diagnostics[0].code, "incomplete-batch");
    }
}
