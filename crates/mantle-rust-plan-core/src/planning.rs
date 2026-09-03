use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::AdmittedWorkspace;
use crate::DependencyRole;
use crate::ExecutionKind;
use crate::PackageFact;
use crate::RustPlan;
use crate::RustPlanBlocker;
use crate::RustPlanCoreError;
use crate::RustPlanReceiptPreimage;
use crate::RustUnitEdge;
use crate::RustUnitPlan;
use crate::TargetFact;
use crate::TargetKind;

const BUILD_MODE: &str = "build";
const TEST_PROFILE: &str = "test";
const DEPENDENCY_PRODUCER_MISSING: &str = "dependency-producer-missing";
const TOPOLOGY_CYCLE: &str = "unit-topology-cycle";

#[derive(Debug, Clone)]
pub(crate) struct UnitDraft {
    pub(crate) unit_id: String,
    pub(crate) package_id: String,
    pub(crate) target: TargetFact,
    pub(crate) execution_kind: ExecutionKind,
    pub(crate) selected_triple: String,
    pub(crate) selected_features: Vec<String>,
}

struct BlockerInput<'a> {
    class: &'a str,
    subject: &'a str,
    detail: &'a str,
}

pub fn plan_workspace(workspace: AdmittedWorkspace) -> Result<RustPlan, RustPlanCoreError> {
    let package_index = package_index(&workspace.packages);
    let drafts = unit_drafts(&workspace, &package_index);
    debug_assert_eq!(package_index.len(), workspace.packages.len());
    debug_assert!(u32::try_from(drafts.len()).is_ok_and(|count| count <= crate::MAX_UNITS));
    let producer_index = library_producer_index(&drafts);
    let (mut edges, mut blockers) = dependency_edges(&workspace, &package_index, &drafts, &producer_index);
    edges.extend(build_script_edges(&drafts));
    edges.sort();
    edges.dedup();
    blockers.sort();
    blockers.dedup();
    let ordered_ids = topology_order(&drafts, &edges);
    if ordered_ids.len() != drafts.len() {
        blockers.push(blocker(BlockerInput {
            class: TOPOLOGY_CYCLE,
            subject: &workspace.workspace_identity,
            detail: "selected Rust unit graph is cyclic",
        }));
    }
    blockers.sort();
    blockers.dedup();
    let units = finalize_units(&workspace, drafts, &edges, &ordered_ids)?;
    build_plan(workspace, units, edges, blockers)
}

fn unit_drafts(workspace: &AdmittedWorkspace, packages: &BTreeMap<String, &PackageFact>) -> Vec<UnitDraft> {
    let mut drafts = Vec::with_capacity(workspace.selected_package_ids.len());
    for package_id in &workspace.selected_package_ids {
        let package = packages[package_id];
        for target in package.targets.iter().filter(|target| target_is_selected(target, &workspace.profile)) {
            drafts.push(unit_draft(workspace, package, target));
        }
    }
    drafts.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    debug_assert!(u32::try_from(drafts.len()).is_ok_and(|count| count <= crate::MAX_UNITS));
    debug_assert!(drafts.iter().all(|draft| !draft.unit_id.is_empty()));
    drafts
}

fn unit_draft(workspace: &AdmittedWorkspace, package: &PackageFact, target: &TargetFact) -> UnitDraft {
    let execution_kind = target.kind.execution_kind();
    let selected_triple = crate::selected_triple(crate::TripleSelectionInput {
        execution_kind: execution_kind.label(),
        host_triple: &workspace.toolchain.host_triple,
        target_triple: &workspace.toolchain.target_triple,
    });
    let selected_features = workspace.selected_features.get(&package.package_id).cloned().unwrap_or_default();
    let dependencies = package
        .dependencies
        .iter()
        .map(|dependency| (dependency.package_id.clone(), dependency.role.label().to_string()))
        .collect();
    let unit_id = crate::native_unit_identity(&crate::NativeUnitIdentityInput {
        package_id: package.package_id.clone(),
        target_name: target.name.clone(),
        target_kind: target.kind.label().to_string(),
        mode: BUILD_MODE.to_string(),
        profile: workspace.profile.clone(),
        source_algorithm: "blake3".to_string(),
        source_value: package.source_identity.clone(),
        features: selected_features.clone(),
        dependencies,
    });
    debug_assert!(!unit_id.is_empty());
    debug_assert!(!selected_triple.is_empty());
    UnitDraft {
        unit_id,
        package_id: package.package_id.clone(),
        target: target.clone(),
        execution_kind,
        selected_triple,
        selected_features,
    }
}

fn dependency_edges(
    workspace: &AdmittedWorkspace,
    packages: &BTreeMap<String, &PackageFact>,
    drafts: &[UnitDraft],
    producers: &BTreeMap<String, String>,
) -> (Vec<RustUnitEdge>, Vec<RustPlanBlocker>) {
    let selected = workspace.selected_package_ids.iter().collect::<BTreeSet<_>>();
    let mut edges = Vec::with_capacity(drafts.len());
    let mut blockers = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let package = packages[&draft.package_id];
        for dependency in
            package.dependencies.iter().filter(|dependency| selected.contains(&dependency.package_id)).filter(
                |dependency| dependency.role != DependencyRole::Development || workspace.profile == TEST_PROFILE,
            )
        {
            match producers.get(&dependency.package_id) {
                Some(producer) => edges.push(RustUnitEdge {
                    producer_unit_id: producer.clone(),
                    consumer_unit_id: draft.unit_id.clone(),
                    role: dependency.role,
                }),
                None => blockers.push(blocker(BlockerInput {
                    class: DEPENDENCY_PRODUCER_MISSING,
                    subject: &dependency.package_id,
                    detail: "selected dependency has no library producer",
                })),
            }
        }
    }
    debug_assert!(edges.len() <= drafts.len().saturating_mul(packages.len()));
    debug_assert!(blockers.len() <= drafts.len().saturating_mul(packages.len()));
    (edges, blockers)
}

fn build_script_edges(drafts: &[UnitDraft]) -> Vec<RustUnitEdge> {
    let build_scripts = drafts
        .iter()
        .filter(|draft| draft.target.kind == TargetKind::BuildScript)
        .map(|draft| (draft.package_id.as_str(), draft.unit_id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let edges = drafts
        .iter()
        .filter_map(|draft| {
            build_scripts.get(draft.package_id.as_str()).filter(|producer| **producer != draft.unit_id).map(
                |producer| RustUnitEdge {
                    producer_unit_id: (*producer).to_string(),
                    consumer_unit_id: draft.unit_id.clone(),
                    role: DependencyRole::Build,
                },
            )
        })
        .collect::<Vec<_>>();
    debug_assert!(edges.len() <= drafts.len());
    debug_assert!(edges.iter().all(|edge| edge.producer_unit_id != edge.consumer_unit_id));
    edges
}

fn topology_order(drafts: &[UnitDraft], edges: &[RustUnitEdge]) -> Vec<String> {
    let mut indegree = drafts.iter().map(|draft| (draft.unit_id.clone(), 0_u32)).collect::<BTreeMap<_, _>>();
    let mut outgoing: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for edge in edges {
        if let Some(value) = indegree.get_mut(&edge.consumer_unit_id) {
            *value = value.saturating_add(1);
        }
        outgoing.entry(edge.producer_unit_id.clone()).or_default().push(edge.consumer_unit_id.clone());
    }
    let mut ready = indegree
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(unit_id, _)| unit_id.clone())
        .collect::<BTreeSet<_>>();
    let mut ordered = Vec::with_capacity(drafts.len());
    while let Some(unit_id) = ready.pop_first() {
        ordered.push(unit_id.clone());
        for consumer in outgoing.get(&unit_id).into_iter().flatten() {
            let Some(count) = indegree.get_mut(consumer) else {
                debug_assert!(false, "topology consumer must be admitted");
                continue;
            };
            *count = count.saturating_sub(1);
            if *count == 0 {
                ready.insert(consumer.clone());
            }
        }
    }
    debug_assert!(ordered.len() <= drafts.len());
    debug_assert!(ordered.iter().all(|unit_id| indegree.contains_key(unit_id)));
    ordered
}

fn finalize_units(
    workspace: &AdmittedWorkspace,
    drafts: Vec<UnitDraft>,
    edges: &[RustUnitEdge],
    ordered_ids: &[String],
) -> Result<Vec<RustUnitPlan>, RustPlanCoreError> {
    let draft_index = drafts.into_iter().map(|draft| (draft.unit_id.clone(), draft)).collect::<BTreeMap<_, _>>();
    let mut units = Vec::with_capacity(ordered_ids.len());
    for (sequence, unit_id) in ordered_ids.iter().enumerate() {
        let sequence =
            u32::try_from(sequence).map_err(|_| RustPlanCoreError::invalid("unit-sequence-width", unit_id.clone()))?;
        let draft = &draft_index[unit_id];
        let dependency_unit_ids = dependency_unit_ids(edges, unit_id);
        let effect = crate::effect_for_unit(workspace, draft, &dependency_unit_ids)?;
        units.push(RustUnitPlan {
            sequence,
            unit_id: draft.unit_id.clone(),
            package_id: draft.package_id.clone(),
            target_name: draft.target.name.clone(),
            target_kind: draft.target.kind,
            crate_name: draft.target.crate_name.clone(),
            execution_kind: draft.execution_kind,
            selected_triple: draft.selected_triple.clone(),
            selected_features: draft.selected_features.clone(),
            dependency_unit_ids,
            effect,
        });
    }
    debug_assert_eq!(units.len(), ordered_ids.len());
    debug_assert!(u32::try_from(units.len()).is_ok_and(|count| count <= crate::MAX_UNITS));
    Ok(units)
}

fn dependency_unit_ids(edges: &[RustUnitEdge], consumer: &str) -> Vec<String> {
    let mut unit_ids = edges
        .iter()
        .filter(|edge| edge.consumer_unit_id == consumer)
        .map(|edge| edge.producer_unit_id.clone())
        .collect::<Vec<_>>();
    unit_ids.sort();
    unit_ids.dedup();
    debug_assert!(unit_ids.len() <= edges.len());
    debug_assert!(unit_ids.iter().all(|unit_id| unit_id != consumer));
    unit_ids
}

fn build_plan(
    workspace: AdmittedWorkspace,
    units: Vec<RustUnitPlan>,
    edges: Vec<RustUnitEdge>,
    blockers: Vec<RustPlanBlocker>,
) -> Result<RustPlan, RustPlanCoreError> {
    let blocker_classes = blockers.iter().map(|blocker| blocker.class.clone()).collect::<Vec<_>>();
    let compatibility = crate::summarize_compatibility(workspace.oracle.is_none(), &blocker_classes);
    let receipt_preimage = RustPlanReceiptPreimage {
        schema: crate::PLAN_RECEIPT_SCHEMA.to_string(),
        workspace_identity: workspace.workspace_identity,
        compiler_identity: workspace.toolchain.compiler_identity,
        compiler_version_identity: workspace.toolchain.compiler_version_identity,
        host_triple: workspace.toolchain.host_triple,
        target_triple: workspace.toolchain.target_triple,
        profile: workspace.profile,
        selected_package_ids: workspace.selected_package_ids,
        selected_features: workspace.selected_features,
        units,
        edges,
        blockers,
        compatibility,
    };
    let receipt_blake3 = crate::identity::plan_digest(&receipt_preimage)?;
    debug_assert_eq!(receipt_blake3.len(), crate::BLAKE3_HEX_CHARS);
    debug_assert_eq!(receipt_preimage.schema, crate::PLAN_RECEIPT_SCHEMA);
    Ok(RustPlan {
        receipt_preimage,
        receipt_blake3,
    })
}

fn library_producer_index(drafts: &[UnitDraft]) -> BTreeMap<String, String> {
    let index = drafts
        .iter()
        .filter(|draft| matches!(draft.target.kind, TargetKind::Library | TargetKind::ProcMacro))
        .map(|draft| (draft.package_id.clone(), draft.unit_id.clone()))
        .collect::<BTreeMap<_, _>>();
    debug_assert!(index.len() <= drafts.len());
    debug_assert!(index.values().all(|unit_id| !unit_id.is_empty()));
    index
}

fn target_is_selected(target: &TargetFact, profile: &str) -> bool {
    target.kind != TargetKind::Test || profile == TEST_PROFILE
}

fn package_index(packages: &[PackageFact]) -> BTreeMap<String, &PackageFact> {
    let index = packages.iter().map(|package| (package.package_id.clone(), package)).collect::<BTreeMap<_, _>>();
    debug_assert_eq!(index.len(), packages.len());
    debug_assert!(!index.is_empty());
    index
}

fn blocker(input: BlockerInput<'_>) -> RustPlanBlocker {
    debug_assert!(!input.class.is_empty());
    debug_assert!(!input.subject.is_empty());
    RustPlanBlocker {
        class: input.class.to_string(),
        subject: input.subject.to_string(),
        detail: input.detail.to_string(),
    }
}

impl DependencyRole {
    const fn label(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Build => "build",
            Self::Development => "development",
        }
    }
}
