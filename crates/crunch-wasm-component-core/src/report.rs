use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Blake3Identity;
use crate::ComponentBlocker;
use crate::DigestError;
use crate::StoreObject;
use crate::blocker::blocker;
use crate::digest::canonical_identity;
use crate::digest::is_count_above_bound;
use crate::digest::is_count_within_bound;
use crate::manifest::REQUIRED_RELEASE_ELIGIBILITY_NON_CLAIM;
use crate::manifest::REQUIRED_RUNTIME_AUTHORITY_NON_CLAIM;

pub const COMPONENT_BUILD_REPORT_SCHEMA: &str = "mantle-wasm-component-build-report-v1";

const MAX_REPORT_NODES: u32 = 64;
const MAX_REPORT_PARENTS: u32 = 32;
const MAX_REPORT_CLAIMS: u32 = 16;
const MAX_REPORT_NON_CLAIMS: u32 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComponentStageKind {
    PackageResolution,
    Lock,
    BindingGeneration,
    Compilation,
    Composition,
    Virtualization,
    MetadataNormalization,
    BuildValidation,
    OctetValidation,
    Wizer,
    Aot,
    MaterializationBundle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComponentStageStatus {
    Planned,
    Succeeded,
    Failed,
    Denied,
    NotRun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BoundedComponentClaim {
    ExactInputIdentities,
    PortableBytesValidated,
    OctetReportBound,
    DenyAllVirtualizationPlanned,
    RepeatedTransformMatched,
    TargetSpecificNativeReceiptBound,
    MaterializationObjectsRehashable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageReportInput {
    pub stage_key: String,
    pub kind: ComponentStageKind,
    pub status: ComponentStageStatus,
    pub parents: Vec<Blake3Identity>,
    pub artifact: Option<StoreObject>,
    pub tool_identity_blake3: Option<Blake3Identity>,
    pub profile_identity_blake3: Option<Blake3Identity>,
    pub claims: Vec<BoundedComponentClaim>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageReportNode {
    pub identity_blake3: Blake3Identity,
    pub stage_key: String,
    pub kind: ComponentStageKind,
    pub status: ComponentStageStatus,
    pub parents: Vec<Blake3Identity>,
    pub artifact: Option<StoreObject>,
    pub tool_identity_blake3: Option<Blake3Identity>,
    pub profile_identity_blake3: Option<Blake3Identity>,
    pub claims: Vec<BoundedComponentClaim>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentBuildReport {
    pub schema: String,
    pub nodes: Vec<StageReportNode>,
    pub non_claims: Vec<String>,
    pub report_identity_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentBuildReportResult {
    pub report: Option<ComponentBuildReport>,
    pub blockers: Vec<ComponentBlocker>,
}

#[derive(Serialize)]
struct ReportIdentityInput {
    schema: String,
    nodes: Vec<StageReportNode>,
    non_claims: Vec<String>,
}

pub fn stage_report_identity(mut input: StageReportInput) -> Result<Blake3Identity, DigestError> {
    input.parents.sort();
    let identity = canonical_identity(input)?;
    debug_assert_eq!(identity.clone().into_hex().len(), crate::BLAKE3_HEX_LENGTH);
    debug_assert!(!identity.clone().into_hex().is_empty());
    Ok(identity)
}

pub fn verify_component_report(report: ComponentBuildReport) -> ComponentBuildReportResult {
    if report.schema != COMPONENT_BUILD_REPORT_SCHEMA {
        return ComponentBuildReportResult {
            report: None,
            blockers: vec![blocker(
                "unsupported-component-report-schema",
                "component-report",
                "component build report schema is unsupported",
            )],
        };
    }
    let expected_identity = report.report_identity_blake3.clone();
    let expected_node_identities: Vec<Blake3Identity> =
        report.nodes.iter().map(|node| node.identity_blake3.clone()).collect();
    let inputs = report.nodes.into_iter().map(stage_input_from_node).collect();
    let rebuilt = build_component_report(inputs, report.non_claims);
    let Some(rebuilt_report) = rebuilt.report else {
        return rebuilt;
    };
    let rebuilt_node_identities: Vec<Blake3Identity> =
        rebuilt_report.nodes.iter().map(|node| node.identity_blake3.clone()).collect();
    if rebuilt_report.report_identity_blake3 != expected_identity || rebuilt_node_identities != expected_node_identities
    {
        return ComponentBuildReportResult {
            report: None,
            blockers: vec![blocker(
                "component-report-identity-mismatch",
                "component-report",
                "component report or stage identities do not match canonical fields",
            )],
        };
    }
    debug_assert_eq!(rebuilt_report.report_identity_blake3, expected_identity);
    debug_assert_eq!(rebuilt_node_identities, expected_node_identities);
    ComponentBuildReportResult {
        report: Some(rebuilt_report),
        blockers: Vec::new(),
    }
}

pub fn build_component_report(
    inputs: Vec<StageReportInput>,
    mut non_claims: Vec<String>,
) -> ComponentBuildReportResult {
    let mut blockers = Vec::new();
    validate_report_bounds(&inputs, &non_claims, &mut blockers);
    let is_input_count_above_bound = is_count_above_bound(inputs.len(), MAX_REPORT_NODES);
    let is_non_claim_count_above_bound = is_count_above_bound(non_claims.len(), MAX_REPORT_NON_CLAIMS);
    if inputs.is_empty() || is_input_count_above_bound || is_non_claim_count_above_bound {
        return ComponentBuildReportResult { report: None, blockers };
    }
    validate_global_non_claims(&non_claims, &mut blockers);
    non_claims.sort();
    if non_claims.windows(crate::ADJACENT_WINDOW_LENGTH).any(|pair| pair[0] == pair[1]) {
        blockers.push(blocker(
            "duplicate-report-non-claim",
            "report.non_claims",
            "component report non-claims contain duplicates",
        ));
    }
    let nodes = build_nodes(inputs, &mut blockers);
    if !blockers.is_empty() {
        return ComponentBuildReportResult { report: None, blockers };
    }
    let identity_input = ReportIdentityInput {
        schema: String::from(COMPONENT_BUILD_REPORT_SCHEMA),
        nodes: nodes.clone(),
        non_claims: non_claims.clone(),
    };
    // A digest is an identity, not a numeric quantity.
    #[allow(tigerstyle::numeric_units)]
    let report_digest = match canonical_identity(identity_input) {
        Ok(identity) => identity,
        Err(_) => {
            return ComponentBuildReportResult {
                report: None,
                blockers: vec![blocker(
                    "component-report-identity-failed",
                    "component-report",
                    "component build report could not be canonically identified",
                )],
            };
        }
    };
    debug_assert!(!nodes.is_empty());
    debug_assert!(is_count_within_bound(nodes.len(), MAX_REPORT_NODES));
    ComponentBuildReportResult {
        report: Some(ComponentBuildReport {
            schema: String::from(COMPONENT_BUILD_REPORT_SCHEMA),
            nodes,
            non_claims,
            report_identity_blake3: report_digest,
        }),
        blockers: Vec::new(),
    }
}

fn build_nodes(inputs: Vec<StageReportInput>, blockers: &mut Vec<ComponentBlocker>) -> Vec<StageReportNode> {
    if is_count_above_bound(inputs.len(), MAX_REPORT_NODES) {
        blockers.push(blocker(
            "component-report-node-limit",
            "component-report",
            "component report stage graph exceeds its fixed bound",
        ));
        return Vec::new();
    }
    let mut stage_keys = BTreeSet::new();
    let mut prior_identities = BTreeSet::new();
    let mut nodes = Vec::with_capacity(inputs.len());
    for mut input in inputs {
        let blocker_count_before = blockers.len();
        validate_stage_input(&input, &stage_keys, &prior_identities, blockers);
        if blockers.len() > blocker_count_before {
            continue;
        }
        input.parents.sort();
        let identity = match stage_report_identity(input.clone()) {
            Ok(identity) => identity,
            Err(_) => {
                blockers.push(blocker(
                    "stage-report-identity-failed",
                    &input.stage_key,
                    "stage report node could not be canonically identified",
                ));
                continue;
            }
        };
        stage_keys.insert(input.stage_key.clone());
        prior_identities.insert(identity.clone());
        nodes.push(stage_node(identity, input));
    }
    debug_assert_eq!(nodes.len(), prior_identities.len());
    debug_assert_eq!(nodes.len(), stage_keys.len());
    nodes
}

fn validate_stage_input(
    input: &StageReportInput,
    stage_keys: &BTreeSet<String>,
    prior_identities: &BTreeSet<Blake3Identity>,
    blockers: &mut Vec<ComponentBlocker>,
) {
    let blocker_count_before = blockers.len();
    if input.stage_key.is_empty() || stage_keys.contains(&input.stage_key) {
        blockers.push(blocker(
            "duplicate-or-empty-stage-key",
            &input.stage_key,
            "stage report keys must be unique and non-empty",
        ));
    }
    if is_count_above_bound(input.parents.len(), MAX_REPORT_PARENTS)
        || is_count_above_bound(input.claims.len(), MAX_REPORT_CLAIMS)
        || is_count_above_bound(input.non_claims.len(), MAX_REPORT_NON_CLAIMS)
    {
        blockers.push(blocker(
            "stage-report-limit",
            &input.stage_key,
            "stage report parents, claims, or non-claims exceed fixed bounds",
        ));
        debug_assert!(blockers.len() > blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return;
    }
    let unique_parents: BTreeSet<Blake3Identity> = input.parents.iter().cloned().collect();
    if unique_parents.len() != input.parents.len() {
        blockers.push(blocker(
            "duplicate-stage-parent",
            &input.stage_key,
            "stage report parent identities contain duplicates",
        ));
    }
    if unique_parents.iter().any(|parent| !prior_identities.contains(parent)) {
        blockers.push(blocker(
            "unknown-or-circular-stage-parent",
            &input.stage_key,
            "stage report parent must identify an earlier node in the same report",
        ));
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_report_bounds(inputs: &[StageReportInput], non_claims: &[String], blockers: &mut Vec<ComponentBlocker>) {
    if inputs.is_empty() || is_count_above_bound(inputs.len(), MAX_REPORT_NODES) {
        blockers.push(blocker(
            "component-report-node-limit",
            "component-report",
            "component report must contain a bounded non-empty stage graph",
        ));
    }
    if is_count_above_bound(non_claims.len(), MAX_REPORT_NON_CLAIMS) {
        blockers.push(blocker(
            "component-report-non-claim-limit",
            "component-report",
            "component report non-claims exceed the fixed bound",
        ));
    }
}

fn validate_global_non_claims(non_claims: &[String], blockers: &mut Vec<ComponentBlocker>) {
    for required in [
        REQUIRED_RUNTIME_AUTHORITY_NON_CLAIM,
        REQUIRED_RELEASE_ELIGIBILITY_NON_CLAIM,
    ] {
        if !non_claims.iter().any(|value| value == required) {
            blockers.push(blocker(
                "missing-report-non-claim",
                required,
                "component report omits a required build-only non-claim",
            ));
        }
    }
}

fn stage_input_from_node(node: StageReportNode) -> StageReportInput {
    StageReportInput {
        stage_key: node.stage_key,
        kind: node.kind,
        status: node.status,
        parents: node.parents,
        artifact: node.artifact,
        tool_identity_blake3: node.tool_identity_blake3,
        profile_identity_blake3: node.profile_identity_blake3,
        claims: node.claims,
        non_claims: node.non_claims,
    }
}

fn stage_node(identity_blake3: Blake3Identity, input: StageReportInput) -> StageReportNode {
    debug_assert!(!input.stage_key.is_empty());
    debug_assert!(is_count_within_bound(input.parents.len(), MAX_REPORT_PARENTS));
    StageReportNode {
        identity_blake3,
        stage_key: input.stage_key,
        kind: input.kind,
        status: input.status,
        parents: input.parents,
        artifact: input.artifact,
        tool_identity_blake3: input.tool_identity_blake3,
        profile_identity_blake3: input.profile_identity_blake3,
        claims: input.claims,
        non_claims: input.non_claims,
    }
}
