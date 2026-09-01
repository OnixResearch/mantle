#![no_std]
//! Pure bounded planning for frontend-neutral castore root composition.

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use serde::Deserialize;
use serde::Serialize;

pub const PLAN_SCHEMA: &str = "mantle-composition-plan-v1";
pub const POLICY_SCHEMA: &str = "mantle-composition-policy-v1";
pub const RECEIPT_SCHEMA: &str = "mantle-composition-receipt-v1";
pub const PLAN_REF_PREFIX: &str = "mantle-composition-plan://blake3/";
pub const POLICY_REF_PREFIX: &str = "mantle-composition-policy://blake3/";
pub const BINDING_REF_PREFIX: &str = "mantle-composition-binding://blake3/";
pub const RECEIPT_REF_PREFIX: &str = "mantle-composition-receipt://blake3/";
pub const MERGE_POLICY_VERSION: u32 = 1;
pub const BLAKE3_HEX_CHARS: usize = 64;
pub const ROOT_DIGEST_BYTES: usize = 32;
pub const ABSOLUTE_BINDINGS_MAX: u32 = 1_024;
pub const ABSOLUTE_COLLISION_DECISIONS_MAX: u32 = 1_024;
pub const ABSOLUTE_ENTRIES_MAX: u32 = 500_001;
pub const ABSOLUTE_DEPTH_MAX: u32 = 128;
pub const ABSOLUTE_PATH_BYTES_MAX: u32 = 4_096;
pub const ABSOLUTE_FILE_BYTES_MAX: u64 = 4_294_967_296;
pub const ABSOLUTE_TOTAL_FILE_BYTES_MAX: u64 = 1_099_511_627_776;
pub const NON_CLAIM: &str = "composition-does-not-prove-abi-closure-runtime-boot-authorization-deployment-or-release";

const PLAN_DOMAIN: &[u8] = b"mantle.composition.plan.v1";
const POLICY_DOMAIN: &[u8] = b"mantle.composition.policy.v1";
const BINDING_DOMAIN: &[u8] = b"mantle.composition.binding.v1";
const RECEIPT_DOMAIN: &[u8] = b"mantle.composition.receipt.v1";
const ROOT_PATH: &str = "";
const PATH_SEPARATOR: char = '/';
const WINDOWS_SEPARATOR: char = '\\';
const WINDOWS_DRIVE_SEPARATOR: char = ':';
const CURRENT_COMPONENT: &str = ".";
const PARENT_COMPONENT: &str = "..";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct CastoreRootRef {
    pub digest_blake3: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CompositionBinding {
    pub root: CastoreRootRef,
    pub mount: String,
    #[serde(default = "no_label", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CollisionDecision {
    pub path: String,
    pub winner_binding_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CompositionPlan {
    pub schema: String,
    pub merge_policy_version: u32,
    pub bindings: Vec<CompositionBinding>,
    #[serde(default = "no_collision_decisions")]
    pub collision_decisions: Vec<CollisionDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RealizationPolicy {
    pub schema: String,
    pub max_bindings: u32,
    pub max_collision_decisions: u32,
    pub max_entries: u32,
    pub max_depth: u32,
    pub max_path_bytes: u32,
    pub max_file_bytes: u64,
    pub max_total_file_bytes: u64,
    pub allow_symlinks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CompositionRequest {
    pub plan: CompositionPlan,
    pub realization_policy: RealizationPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NormalizedBinding {
    pub binding_ref: String,
    pub root: CastoreRootRef,
    pub mount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PreparedComposition {
    pub plan_ref: String,
    pub realization_policy_ref: String,
    pub merge_policy_version: u32,
    pub bindings: Vec<NormalizedBinding>,
    pub collision_decisions: Vec<CollisionDecision>,
    pub realization_policy: RealizationPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotEntry {
    pub name: String,
    pub node: SnapshotNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotNode {
    Directory {
        entries: Vec<SnapshotEntry>,
    },
    File {
        digest_blake3: String,
        size_bytes: u64,
        executable: bool,
    },
    Symlink {
        target: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootSnapshot {
    pub root: CastoreRootRef,
    pub node: SnapshotNode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MergeOutcomeKind {
    DirectoryUnion,
    IdenticalLeafDeduplicated,
    ExplicitReplacement,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MergeOutcome {
    pub path: String,
    pub kind: MergeOutcomeKind,
    #[serde(default = "no_winner_binding_ref", skip_serializing_if = "Option::is_none")]
    pub winner_binding_ref: Option<String>,
    #[serde(default = "no_displaced_binding_refs")]
    pub displaced_binding_refs: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LimitUsage {
    pub bindings: u32,
    pub collision_decisions: u32,
    pub entries: u32,
    pub depth: u32,
    pub path_bytes: u32,
    pub largest_file_bytes: u64,
    pub total_file_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositionOutcome {
    pub root: SnapshotNode,
    pub merge_outcomes: Vec<MergeOutcome>,
    pub limit_usage: LimitUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RealizationReceipt {
    pub schema: String,
    pub receipt_ref: String,
    pub plan_ref: String,
    pub realization_policy_ref: String,
    pub merge_policy_version: u32,
    pub input_roots: Vec<CastoreRootRef>,
    pub merge_outcomes: Vec<MergeOutcome>,
    pub limit_usage: LimitUsage,
    pub resulting_root: CastoreRootRef,
    pub unsupported_metadata_classes: Vec<String>,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositionError {
    InvalidPlanSchema,
    InvalidPolicySchema,
    UnsupportedMergePolicy,
    InvalidRootRef,
    InvalidMountPath(String),
    InvalidCollisionPath(String),
    InvalidBindingRef,
    EmptyBindings,
    DuplicateBinding(String),
    DuplicateDecision(String),
    LimitInvalid(&'static str),
    LimitExceeded(&'static str),
    MissingSnapshot(String),
    UnexpectedSnapshot(String),
    SnapshotRootMismatch(String),
    RootNotDirectory(String),
    InvalidEntryName(String),
    DuplicateEntry(String),
    SymlinkRejected(String),
    UnresolvedCollision(String),
    NonContributingWinner(String),
    StaleDecision(String),
    IntegerOverflow(&'static str),
}

impl fmt::Display for CompositionError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPlanSchema => output.write_str("composition-invalid-plan-schema"),
            Self::InvalidPolicySchema => output.write_str("composition-invalid-policy-schema"),
            Self::UnsupportedMergePolicy => output.write_str("composition-unsupported-merge-policy"),
            Self::InvalidRootRef => output.write_str("composition-invalid-root-ref"),
            Self::InvalidMountPath(path) => write!(output, "composition-invalid-mount-path:{path}"),
            Self::InvalidCollisionPath(path) => write!(output, "composition-invalid-collision-path:{path}"),
            Self::InvalidBindingRef => output.write_str("composition-invalid-binding-ref"),
            Self::EmptyBindings => output.write_str("composition-empty-bindings"),
            Self::DuplicateBinding(binding_ref) => write!(output, "composition-duplicate-binding:{binding_ref}"),
            Self::DuplicateDecision(path) => write!(output, "composition-duplicate-decision:{path}"),
            Self::LimitInvalid(limit) => write!(output, "composition-invalid-limit:{limit}"),
            Self::LimitExceeded(limit) => write!(output, "composition-limit-exceeded:{limit}"),
            Self::MissingSnapshot(root) => write!(output, "composition-missing-snapshot:{root}"),
            Self::UnexpectedSnapshot(root) => write!(output, "composition-unexpected-snapshot:{root}"),
            Self::SnapshotRootMismatch(root) => write!(output, "composition-snapshot-root-mismatch:{root}"),
            Self::RootNotDirectory(root) => write!(output, "composition-root-not-directory:{root}"),
            Self::InvalidEntryName(path) => write!(output, "composition-invalid-entry-name:{path}"),
            Self::DuplicateEntry(path) => write!(output, "composition-duplicate-entry:{path}"),
            Self::SymlinkRejected(path) => write!(output, "composition-symlink-rejected:{path}"),
            Self::UnresolvedCollision(path) => write!(output, "composition-unresolved-collision:{path}"),
            Self::NonContributingWinner(path) => write!(output, "composition-non-contributing-winner:{path}"),
            Self::StaleDecision(path) => write!(output, "composition-stale-decision:{path}"),
            Self::IntegerOverflow(field) => write!(output, "composition-integer-overflow:{field}"),
        }
    }
}

#[derive(Clone)]
struct Contribution {
    binding_ref: String,
    node: SnapshotNode,
}

fn no_label() -> Option<String> {
    None
}

fn no_collision_decisions() -> Vec<CollisionDecision> {
    Vec::new()
}

fn no_winner_binding_ref() -> Option<String> {
    None
}

fn no_displaced_binding_refs() -> Vec<String> {
    Vec::new()
}

pub fn prepare_composition(request: &CompositionRequest) -> Result<PreparedComposition, CompositionError> {
    validate_policy(&request.realization_policy)?;
    if request.plan.schema != PLAN_SCHEMA {
        return Err(CompositionError::InvalidPlanSchema);
    }
    if request.plan.merge_policy_version != MERGE_POLICY_VERSION {
        return Err(CompositionError::UnsupportedMergePolicy);
    }
    if request.plan.bindings.is_empty() {
        return Err(CompositionError::EmptyBindings);
    }
    check_count(request.plan.bindings.len(), request.realization_policy.max_bindings, "bindings")?;
    check_count(
        request.plan.collision_decisions.len(),
        request.realization_policy.max_collision_decisions,
        "collision-decisions",
    )?;

    let mut bindings = request
        .plan
        .bindings
        .iter()
        .map(|binding| normalize_binding(binding, &request.realization_policy))
        .collect::<Result<Vec<_>, _>>()?;
    bindings.sort_by(|left, right| left.binding_ref.cmp(&right.binding_ref));
    reject_duplicate_bindings(&bindings)?;

    let mut decisions = request
        .plan
        .collision_decisions
        .iter()
        .map(|decision| normalize_decision(decision, &request.realization_policy))
        .collect::<Result<Vec<_>, _>>()?;
    decisions.sort_by(|left, right| left.path.cmp(&right.path));
    reject_duplicate_decisions(&decisions)?;

    let plan_ref = hash_plan(request.plan.merge_policy_version, &bindings, &decisions)?;
    let realization_policy_ref = hash_policy(&request.realization_policy)?;
    let prepared = PreparedComposition {
        plan_ref,
        realization_policy_ref,
        merge_policy_version: request.plan.merge_policy_version,
        bindings,
        collision_decisions: decisions,
        realization_policy: request.realization_policy.clone(),
    };
    debug_assert!(!prepared.bindings.is_empty());
    debug_assert!(
        u32::try_from(prepared.bindings.len()).is_ok_and(|count| count <= request.realization_policy.max_bindings)
    );
    Ok(prepared)
}

pub fn plan_composition(
    prepared: &PreparedComposition,
    snapshots: Vec<RootSnapshot>,
) -> Result<CompositionOutcome, CompositionError> {
    let snapshots_by_root = index_snapshots(prepared, snapshots)?;
    let mut usage = LimitUsage {
        bindings: count_u32(prepared.bindings.len(), "bindings")?,
        collision_decisions: count_u32(prepared.collision_decisions.len(), "collision-decisions")?,
        entries: 0,
        depth: 0,
        path_bytes: 0,
        largest_file_bytes: 0,
        total_file_bytes: 0,
    };
    let mut contributions = Vec::with_capacity(prepared.bindings.len());
    for binding in &prepared.bindings {
        let snapshot = snapshots_by_root
            .get(&binding.root)
            .ok_or_else(|| CompositionError::MissingSnapshot(binding.root.digest_blake3.clone()))?;
        if !matches!(snapshot.node, SnapshotNode::Directory { .. }) {
            return Err(CompositionError::RootNotDirectory(binding.root.digest_blake3.clone()));
        }
        let mounted = mount_snapshot(&binding.mount, snapshot.node.clone());
        validate_snapshot(&mounted, ROOT_PATH, 0, &prepared.realization_policy, &mut usage)?;
        contributions.push(Contribution {
            binding_ref: binding.binding_ref.clone(),
            node: mounted,
        });
    }

    let decisions = prepared
        .collision_decisions
        .iter()
        .map(|decision| (decision.path.clone(), decision.winner_binding_ref.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut used_decisions = BTreeSet::new();
    let mut outcomes = Vec::new();
    let root = merge_contributions(ROOT_PATH, contributions, &decisions, &mut used_decisions, &mut outcomes)?;
    for decision in decisions.keys() {
        if !used_decisions.contains(decision) {
            return Err(CompositionError::StaleDecision(decision.clone()));
        }
    }
    outcomes.sort_by(|left, right| left.path.cmp(&right.path));
    let outcome = CompositionOutcome {
        root,
        merge_outcomes: outcomes,
        limit_usage: usage,
    };
    debug_assert_eq!(outcome.limit_usage.bindings, count_u32(prepared.bindings.len(), "bindings")?);
    debug_assert!(outcome.limit_usage.entries <= prepared.realization_policy.max_entries);
    Ok(outcome)
}

pub fn finalize_receipt(
    prepared: &PreparedComposition,
    outcome: &CompositionOutcome,
    resulting_root: CastoreRootRef,
) -> Result<RealizationReceipt, CompositionError> {
    validate_root_ref(&resulting_root)?;
    let mut input_roots = prepared.bindings.iter().map(|binding| binding.root.clone()).collect::<Vec<_>>();
    input_roots.sort();
    input_roots.dedup();
    let non_claim_labels = vec_of_strings(&[
        "ownership",
        "acl",
        "capability",
        "security-xattr",
        "hard-link",
        "device-node",
    ]);
    let receipt_ref = hash_receipt(prepared, outcome, &input_roots, &resulting_root, &non_claim_labels)?;
    let receipt = RealizationReceipt {
        schema: RECEIPT_SCHEMA.to_string(),
        receipt_ref,
        plan_ref: prepared.plan_ref.clone(),
        realization_policy_ref: prepared.realization_policy_ref.clone(),
        merge_policy_version: prepared.merge_policy_version,
        input_roots,
        merge_outcomes: outcome.merge_outcomes.clone(),
        limit_usage: outcome.limit_usage.clone(),
        resulting_root,
        unsupported_metadata_classes: non_claim_labels,
        non_claim: NON_CLAIM.to_string(),
    };
    debug_assert_eq!(receipt.plan_ref, prepared.plan_ref);
    debug_assert!(!receipt.unsupported_metadata_classes.is_empty());
    Ok(receipt)
}

fn validate_policy(policy: &RealizationPolicy) -> Result<(), CompositionError> {
    if policy.schema != POLICY_SCHEMA {
        return Err(CompositionError::InvalidPolicySchema);
    }
    let required_u32_bounds = [
        (policy.max_bindings, "bindings"),
        (policy.max_collision_decisions, "collision-decisions"),
        (policy.max_entries, "entries"),
        (policy.max_depth, "depth"),
        (policy.max_path_bytes, "path-bytes"),
    ];
    for (value, name) in required_u32_bounds {
        if value == 0 {
            return Err(CompositionError::LimitInvalid(name));
        }
    }
    if policy.max_file_bytes == 0 {
        return Err(CompositionError::LimitInvalid("file-bytes"));
    }
    if policy.max_total_file_bytes == 0 {
        return Err(CompositionError::LimitInvalid("total-file-bytes"));
    }
    let absolute_u32_bounds = [
        (policy.max_bindings, ABSOLUTE_BINDINGS_MAX, "bindings"),
        (policy.max_collision_decisions, ABSOLUTE_COLLISION_DECISIONS_MAX, "collision-decisions"),
        (policy.max_entries, ABSOLUTE_ENTRIES_MAX, "entries"),
        (policy.max_depth, ABSOLUTE_DEPTH_MAX, "depth"),
        (policy.max_path_bytes, ABSOLUTE_PATH_BYTES_MAX, "path-bytes"),
    ];
    for (value, absolute_max, name) in absolute_u32_bounds {
        if value > absolute_max {
            return Err(CompositionError::LimitExceeded(name));
        }
    }
    if policy.max_file_bytes > ABSOLUTE_FILE_BYTES_MAX {
        return Err(CompositionError::LimitExceeded("file-bytes"));
    }
    if policy.max_total_file_bytes > ABSOLUTE_TOTAL_FILE_BYTES_MAX {
        return Err(CompositionError::LimitExceeded("total-file-bytes"));
    }
    debug_assert!(policy.max_bindings <= ABSOLUTE_BINDINGS_MAX);
    debug_assert!(policy.max_total_file_bytes <= ABSOLUTE_TOTAL_FILE_BYTES_MAX);
    Ok(())
}

fn normalize_binding(
    binding: &CompositionBinding,
    policy: &RealizationPolicy,
) -> Result<NormalizedBinding, CompositionError> {
    validate_root_ref(&binding.root)?;
    let mount =
        normalize_path(&binding.mount, true).map_err(|_| CompositionError::InvalidMountPath(binding.mount.clone()))?;
    check_path_bound(&mount, policy.max_path_bytes, "path-bytes")?;
    let binding_ref = hash_binding(&binding.root, &mount)?;
    Ok(NormalizedBinding {
        binding_ref,
        root: binding.root.clone(),
        mount,
    })
}

fn normalize_decision(
    decision: &CollisionDecision,
    policy: &RealizationPolicy,
) -> Result<CollisionDecision, CompositionError> {
    let path = normalize_path(&decision.path, true)
        .map_err(|_| CompositionError::InvalidCollisionPath(decision.path.clone()))?;
    check_path_bound(&path, policy.max_path_bytes, "path-bytes")?;
    if !is_binding_ref(&decision.winner_binding_ref) {
        return Err(CompositionError::InvalidBindingRef);
    }
    Ok(CollisionDecision {
        path,
        winner_binding_ref: decision.winner_binding_ref.clone(),
    })
}

fn normalize_path(path: &str, is_root_allowed: bool) -> Result<String, ()> {
    if path.is_empty() {
        return if is_root_allowed { Ok(String::new()) } else { Err(()) };
    }
    if path.starts_with(PATH_SEPARATOR) || path.starts_with(WINDOWS_SEPARATOR) {
        return Err(());
    }
    if path.ends_with(PATH_SEPARATOR) || path.contains(WINDOWS_SEPARATOR) {
        return Err(());
    }
    if path.contains("//") {
        return Err(());
    }
    if path.as_bytes().get(1).copied() == Some(WINDOWS_DRIVE_SEPARATOR as u8) {
        return Err(());
    }
    let component_count = path.split(PATH_SEPARATOR).count();
    let mut normalized = Vec::with_capacity(component_count);
    for component in path.split(PATH_SEPARATOR) {
        if component.is_empty() || component == CURRENT_COMPONENT || component == PARENT_COMPONENT {
            return Err(());
        }
        normalized.push(component);
    }
    debug_assert_eq!(normalized.len(), component_count);
    debug_assert!(normalized.iter().all(|component| !component.is_empty()));
    Ok(normalized.join("/"))
}

fn validate_root_ref(root: &CastoreRootRef) -> Result<(), CompositionError> {
    if root.digest_blake3.len() != BLAKE3_HEX_CHARS
        || !root.digest_blake3.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(CompositionError::InvalidRootRef);
    }
    Ok(())
}

fn reject_duplicate_bindings(bindings: &[NormalizedBinding]) -> Result<(), CompositionError> {
    for pair in bindings.windows(2) {
        if pair[0].binding_ref == pair[1].binding_ref {
            return Err(CompositionError::DuplicateBinding(pair[0].binding_ref.clone()));
        }
    }
    Ok(())
}

fn reject_duplicate_decisions(decisions: &[CollisionDecision]) -> Result<(), CompositionError> {
    for pair in decisions.windows(2) {
        if pair[0].path == pair[1].path {
            return Err(CompositionError::DuplicateDecision(pair[0].path.clone()));
        }
    }
    Ok(())
}

fn index_snapshots(
    prepared: &PreparedComposition,
    snapshots: Vec<RootSnapshot>,
) -> Result<BTreeMap<CastoreRootRef, RootSnapshot>, CompositionError> {
    let expected = prepared.bindings.iter().map(|binding| binding.root.clone()).collect::<BTreeSet<_>>();
    let mut indexed = BTreeMap::new();
    for snapshot in snapshots {
        validate_root_ref(&snapshot.root)?;
        if !expected.contains(&snapshot.root) {
            return Err(CompositionError::UnexpectedSnapshot(snapshot.root.digest_blake3));
        }
        let root = snapshot.root.clone();
        if indexed.insert(root.clone(), snapshot).is_some() {
            return Err(CompositionError::SnapshotRootMismatch(root.digest_blake3));
        }
    }
    for root in expected {
        if !indexed.contains_key(&root) {
            return Err(CompositionError::MissingSnapshot(root.digest_blake3));
        }
    }
    debug_assert_eq!(indexed.len(), prepared.bindings.len());
    debug_assert!(indexed.keys().all(|root| prepared.bindings.iter().any(|binding| &binding.root == root)));
    Ok(indexed)
}

fn validate_snapshot(
    node: &SnapshotNode,
    path: &str,
    depth: u32,
    policy: &RealizationPolicy,
    usage: &mut LimitUsage,
) -> Result<(), CompositionError> {
    usage.depth = usage.depth.max(depth);
    if usage.depth > policy.max_depth {
        return Err(CompositionError::LimitExceeded("depth"));
    }
    usage.path_bytes = usage.path_bytes.max(count_u32(path.len(), "path-bytes")?);
    if usage.path_bytes > policy.max_path_bytes {
        return Err(CompositionError::LimitExceeded("path-bytes"));
    }
    match node {
        SnapshotNode::Directory { entries } => validate_directory(entries, path, depth, policy, usage)?,
        SnapshotNode::File {
            digest_blake3,
            size_bytes,
            ..
        } => {
            validate_digest(digest_blake3)?;
            usage.largest_file_bytes = usage.largest_file_bytes.max(*size_bytes);
            if usage.largest_file_bytes > policy.max_file_bytes {
                return Err(CompositionError::LimitExceeded("file-bytes"));
            }
            usage.total_file_bytes = usage
                .total_file_bytes
                .checked_add(*size_bytes)
                .ok_or(CompositionError::IntegerOverflow("total-file-bytes"))?;
            if usage.total_file_bytes > policy.max_total_file_bytes {
                return Err(CompositionError::LimitExceeded("total-file-bytes"));
            }
        }
        SnapshotNode::Symlink { .. } => {
            if !policy.allow_symlinks {
                return Err(CompositionError::SymlinkRejected(path.to_string()));
            }
        }
    }
    debug_assert!(usage.depth <= policy.max_depth);
    debug_assert!(usage.path_bytes <= policy.max_path_bytes);
    Ok(())
}

fn validate_directory(
    entries: &[SnapshotEntry],
    path: &str,
    depth: u32,
    policy: &RealizationPolicy,
    usage: &mut LimitUsage,
) -> Result<(), CompositionError> {
    let mut names = BTreeSet::new();
    for entry in entries {
        if normalize_path(&entry.name, false).is_err() || entry.name.contains(PATH_SEPARATOR) {
            return Err(CompositionError::InvalidEntryName(join_path(PathJoin {
                parent: path,
                child: &entry.name,
            })));
        }
        if !names.insert(entry.name.clone()) {
            return Err(CompositionError::DuplicateEntry(join_path(PathJoin {
                parent: path,
                child: &entry.name,
            })));
        }
        usage.entries = usage.entries.checked_add(1).ok_or(CompositionError::IntegerOverflow("entries"))?;
        if usage.entries > policy.max_entries {
            return Err(CompositionError::LimitExceeded("entries"));
        }
        let child_path = join_path(PathJoin {
            parent: path,
            child: &entry.name,
        });
        let child_depth = depth.checked_add(1).ok_or(CompositionError::IntegerOverflow("depth"))?;
        validate_snapshot(&entry.node, &child_path, child_depth, policy, usage)?;
    }
    debug_assert_eq!(names.len(), entries.len());
    debug_assert!(usage.entries <= policy.max_entries);
    Ok(())
}

fn mount_snapshot(mount: &str, mut node: SnapshotNode) -> SnapshotNode {
    if mount.is_empty() {
        return node;
    }
    let components = mount.split(PATH_SEPARATOR).collect::<Vec<_>>();
    for component in components.into_iter().rev() {
        node = SnapshotNode::Directory {
            entries: vec![SnapshotEntry {
                name: component.to_string(),
                node,
            }],
        };
    }
    node
}

fn merge_contributions(
    path: &str,
    contributions: Vec<Contribution>,
    decisions: &BTreeMap<String, String>,
    used_decisions: &mut BTreeSet<String>,
    outcomes: &mut Vec<MergeOutcome>,
) -> Result<SnapshotNode, CompositionError> {
    if contributions.iter().all(|item| matches!(item.node, SnapshotNode::Directory { .. })) {
        return merge_directories(path, contributions, decisions, used_decisions, outcomes);
    }
    let first = &contributions[0].node;
    if contributions.iter().all(|item| item.node == *first) {
        if contributions.len() > 1 {
            outcomes.push(MergeOutcome {
                path: path.to_string(),
                kind: MergeOutcomeKind::IdenticalLeafDeduplicated,
                winner_binding_ref: None,
                displaced_binding_refs: Vec::new(),
            });
        }
        return Ok(first.clone());
    }
    resolve_collision(path, contributions, decisions, used_decisions, outcomes)
}

fn merge_directories(
    path: &str,
    contributions: Vec<Contribution>,
    decisions: &BTreeMap<String, String>,
    used_decisions: &mut BTreeSet<String>,
    outcomes: &mut Vec<MergeOutcome>,
) -> Result<SnapshotNode, CompositionError> {
    debug_assert!(!contributions.is_empty());
    debug_assert!(contributions.iter().all(|item| matches!(item.node, SnapshotNode::Directory { .. })));
    let contributor_count = contributions.len();
    let mut children = BTreeMap::<String, Vec<Contribution>>::new();
    for contribution in contributions {
        let SnapshotNode::Directory { entries } = contribution.node else {
            return Err(CompositionError::UnresolvedCollision(path.to_string()));
        };
        for entry in entries {
            children.entry(entry.name).or_default().push(Contribution {
                binding_ref: contribution.binding_ref.clone(),
                node: entry.node,
            });
        }
    }
    let mut entries = Vec::with_capacity(children.len());
    for (name, child_contributions) in children {
        let child_path = join_path(PathJoin {
            parent: path,
            child: &name,
        });
        entries.push(SnapshotEntry {
            name,
            node: merge_contributions(&child_path, child_contributions, decisions, used_decisions, outcomes)?,
        });
    }
    if contributor_count > 1 {
        outcomes.push(MergeOutcome {
            path: path.to_string(),
            kind: MergeOutcomeKind::DirectoryUnion,
            winner_binding_ref: None,
            displaced_binding_refs: Vec::new(),
        });
    }
    Ok(SnapshotNode::Directory { entries })
}

fn resolve_collision(
    path: &str,
    contributions: Vec<Contribution>,
    decisions: &BTreeMap<String, String>,
    used_decisions: &mut BTreeSet<String>,
    outcomes: &mut Vec<MergeOutcome>,
) -> Result<SnapshotNode, CompositionError> {
    debug_assert!(!contributions.is_empty());
    debug_assert!(contributions.iter().any(|item| !matches!(item.node, SnapshotNode::Directory { .. })));
    let winner_ref = decisions.get(path).ok_or_else(|| CompositionError::UnresolvedCollision(path.to_string()))?;
    let winner = contributions
        .iter()
        .find(|item| item.binding_ref == *winner_ref)
        .ok_or_else(|| CompositionError::NonContributingWinner(path.to_string()))?;
    used_decisions.insert(path.to_string());
    let mut displaced = contributions
        .iter()
        .filter(|item| item.binding_ref != *winner_ref)
        .map(|item| item.binding_ref.clone())
        .collect::<Vec<_>>();
    displaced.sort();
    displaced.dedup();
    outcomes.push(MergeOutcome {
        path: path.to_string(),
        kind: MergeOutcomeKind::ExplicitReplacement,
        winner_binding_ref: Some(winner_ref.clone()),
        displaced_binding_refs: displaced,
    });
    Ok(winner.node.clone())
}

fn hash_binding(root: &CastoreRootRef, mount: &str) -> Result<String, CompositionError> {
    let mut hasher = blake3::Hasher::new();
    hash_bytes(&mut hasher, BINDING_DOMAIN)?;
    hash_root(&mut hasher, root)?;
    hash_bytes(&mut hasher, mount.as_bytes())?;
    Ok(format!("{BINDING_REF_PREFIX}{}", hasher.finalize().to_hex()))
}

fn hash_plan(
    merge_policy_version: u32,
    bindings: &[NormalizedBinding],
    decisions: &[CollisionDecision],
) -> Result<String, CompositionError> {
    let mut hasher = blake3::Hasher::new();
    hash_bytes(&mut hasher, PLAN_DOMAIN)?;
    hasher.update(&merge_policy_version.to_le_bytes());
    hash_count(&mut hasher, bindings.len())?;
    for binding in bindings {
        hash_bytes(&mut hasher, binding.binding_ref.as_bytes())?;
        hash_root(&mut hasher, &binding.root)?;
        hash_bytes(&mut hasher, binding.mount.as_bytes())?;
    }
    hash_count(&mut hasher, decisions.len())?;
    for decision in decisions {
        hash_bytes(&mut hasher, decision.path.as_bytes())?;
        hash_bytes(&mut hasher, decision.winner_binding_ref.as_bytes())?;
    }
    Ok(format!("{PLAN_REF_PREFIX}{}", hasher.finalize().to_hex()))
}

fn hash_policy(policy: &RealizationPolicy) -> Result<String, CompositionError> {
    let mut hasher = blake3::Hasher::new();
    hash_bytes(&mut hasher, POLICY_DOMAIN)?;
    hasher.update(&policy.max_bindings.to_le_bytes());
    hasher.update(&policy.max_collision_decisions.to_le_bytes());
    hasher.update(&policy.max_entries.to_le_bytes());
    hasher.update(&policy.max_depth.to_le_bytes());
    hasher.update(&policy.max_path_bytes.to_le_bytes());
    hasher.update(&policy.max_file_bytes.to_le_bytes());
    hasher.update(&policy.max_total_file_bytes.to_le_bytes());
    hasher.update(&[u8::from(policy.allow_symlinks)]);
    Ok(format!("{POLICY_REF_PREFIX}{}", hasher.finalize().to_hex()))
}

fn hash_receipt(
    prepared: &PreparedComposition,
    outcome: &CompositionOutcome,
    input_roots: &[CastoreRootRef],
    result: &CastoreRootRef,
    unsupported: &[String],
) -> Result<String, CompositionError> {
    debug_assert!(input_roots.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert!(!unsupported.is_empty());
    let mut hasher = blake3::Hasher::new();
    hash_bytes(&mut hasher, RECEIPT_DOMAIN)?;
    hash_bytes(&mut hasher, prepared.plan_ref.as_bytes())?;
    hash_bytes(&mut hasher, prepared.realization_policy_ref.as_bytes())?;
    hasher.update(&prepared.merge_policy_version.to_le_bytes());
    hash_count(&mut hasher, input_roots.len())?;
    for root in input_roots {
        hash_root(&mut hasher, root)?;
    }
    hash_count(&mut hasher, outcome.merge_outcomes.len())?;
    for merge in &outcome.merge_outcomes {
        hash_bytes(&mut hasher, merge.path.as_bytes())?;
        hash_bytes(&mut hasher, merge_kind(merge.kind.clone()).as_bytes())?;
        hash_optional(&mut hasher, merge.winner_binding_ref.as_deref())?;
        hash_count(&mut hasher, merge.displaced_binding_refs.len())?;
        for displaced in &merge.displaced_binding_refs {
            hash_bytes(&mut hasher, displaced.as_bytes())?;
        }
    }
    hash_usage(&mut hasher, &outcome.limit_usage);
    hash_root(&mut hasher, result)?;
    hash_count(&mut hasher, unsupported.len())?;
    for item in unsupported {
        hash_bytes(&mut hasher, item.as_bytes())?;
    }
    hash_bytes(&mut hasher, NON_CLAIM.as_bytes())?;
    Ok(format!("{RECEIPT_REF_PREFIX}{}", hasher.finalize().to_hex()))
}

fn hash_usage(hasher: &mut blake3::Hasher, usage: &LimitUsage) {
    hasher.update(&usage.bindings.to_le_bytes());
    hasher.update(&usage.collision_decisions.to_le_bytes());
    hasher.update(&usage.entries.to_le_bytes());
    hasher.update(&usage.depth.to_le_bytes());
    hasher.update(&usage.path_bytes.to_le_bytes());
    hasher.update(&usage.largest_file_bytes.to_le_bytes());
    hasher.update(&usage.total_file_bytes.to_le_bytes());
}

fn hash_root(hasher: &mut blake3::Hasher, root: &CastoreRootRef) -> Result<(), CompositionError> {
    hash_bytes(hasher, root.digest_blake3.as_bytes())?;
    hasher.update(&root.size.to_le_bytes());
    Ok(())
}

fn hash_optional(hasher: &mut blake3::Hasher, value: Option<&str>) -> Result<(), CompositionError> {
    match value {
        Some(value) => {
            hasher.update(&[1]);
            hash_bytes(hasher, value.as_bytes())
        }
        None => {
            hasher.update(&[0]);
            Ok(())
        }
    }
}

fn hash_count(hasher: &mut blake3::Hasher, count: usize) -> Result<(), CompositionError> {
    let count = count_u32(count, "hash-count")?;
    hasher.update(&count.to_le_bytes());
    Ok(())
}

fn hash_bytes(hasher: &mut blake3::Hasher, bytes: &[u8]) -> Result<(), CompositionError> {
    hash_count(hasher, bytes.len())?;
    hasher.update(bytes);
    Ok(())
}

fn validate_digest(digest: &str) -> Result<(), CompositionError> {
    validate_root_ref(&CastoreRootRef {
        digest_blake3: digest.to_string(),
        size: 0,
    })
}

fn check_count(count: usize, limit: u32, name: &'static str) -> Result<(), CompositionError> {
    if count_u32(count, name)? > limit {
        return Err(CompositionError::LimitExceeded(name));
    }
    Ok(())
}

fn check_path_bound(path: &str, limit: u32, name: &'static str) -> Result<(), CompositionError> {
    check_count(path.len(), limit, name)
}

fn count_u32(value: usize, field: &'static str) -> Result<u32, CompositionError> {
    u32::try_from(value).map_err(|_| CompositionError::IntegerOverflow(field))
}

fn is_binding_ref(value: &str) -> bool {
    value.strip_prefix(BINDING_REF_PREFIX).is_some_and(|digest| {
        digest.len() == BLAKE3_HEX_CHARS
            && digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

struct PathJoin<'a> {
    parent: &'a str,
    child: &'a str,
}

fn join_path(parts: PathJoin<'_>) -> String {
    if parts.parent.is_empty() {
        parts.child.to_string()
    } else {
        format!("{}/{}", parts.parent, parts.child)
    }
}

fn merge_kind(kind: MergeOutcomeKind) -> &'static str {
    match kind {
        MergeOutcomeKind::DirectoryUnion => "directory-union",
        MergeOutcomeKind::IdenticalLeafDeduplicated => "identical-leaf-deduplicated",
        MergeOutcomeKind::ExplicitReplacement => "explicit-replacement",
    }
}

fn vec_of_strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    const FILE_SIZE_BYTES: u64 = 7;
    const DEFAULT_LIMIT: u32 = 64;
    const DEFAULT_FILE_LIMIT_BYTES: u64 = 1_024;
    const ROOT_A_HEX: &str = "1111111111111111111111111111111111111111111111111111111111111111";
    const ROOT_B_HEX: &str = "2222222222222222222222222222222222222222222222222222222222222222";
    const FILE_A_HEX: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const FILE_B_HEX: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn root(digest: &str) -> CastoreRootRef {
        CastoreRootRef {
            digest_blake3: digest.to_string(),
            size: 1,
        }
    }

    fn policy() -> RealizationPolicy {
        RealizationPolicy {
            schema: POLICY_SCHEMA.to_string(),
            max_bindings: DEFAULT_LIMIT,
            max_collision_decisions: DEFAULT_LIMIT,
            max_entries: DEFAULT_LIMIT,
            max_depth: DEFAULT_LIMIT,
            max_path_bytes: DEFAULT_LIMIT,
            max_file_bytes: DEFAULT_FILE_LIMIT_BYTES,
            max_total_file_bytes: DEFAULT_FILE_LIMIT_BYTES,
            allow_symlinks: true,
        }
    }

    fn binding(digest: &str, mount: &str, label: &str) -> CompositionBinding {
        CompositionBinding {
            root: root(digest),
            mount: mount.to_string(),
            label: Some(label.to_string()),
        }
    }

    fn request(bindings: Vec<CompositionBinding>) -> CompositionRequest {
        CompositionRequest {
            plan: CompositionPlan {
                schema: PLAN_SCHEMA.to_string(),
                merge_policy_version: MERGE_POLICY_VERSION,
                bindings,
                collision_decisions: Vec::new(),
            },
            realization_policy: policy(),
        }
    }

    fn file(digest: &str) -> SnapshotNode {
        SnapshotNode::File {
            digest_blake3: digest.to_string(),
            size_bytes: FILE_SIZE_BYTES,
            executable: false,
        }
    }

    fn directory(entries: &[(&str, SnapshotNode)]) -> SnapshotNode {
        SnapshotNode::Directory {
            entries: entries
                .iter()
                .map(|(name, node)| SnapshotEntry {
                    name: (*name).to_string(),
                    node: node.clone(),
                })
                .collect(),
        }
    }

    fn snapshot(digest: &str, node: SnapshotNode) -> RootSnapshot {
        RootSnapshot {
            root: root(digest),
            node,
        }
    }

    #[test]
    fn plan_identity_ignores_binding_order_and_labels() {
        let first =
            prepare_composition(&request(vec![binding(ROOT_A_HEX, "usr", "one"), binding(ROOT_B_HEX, "", "two")]))
                .unwrap();
        let second = prepare_composition(&request(vec![
            binding(ROOT_B_HEX, "", "changed"),
            binding(ROOT_A_HEX, "usr", "other"),
        ]))
        .unwrap();

        assert_eq!(first, second);
        assert_eq!(first.bindings.len(), 2);
        assert!(first.bindings.iter().all(|binding| !binding.binding_ref.is_empty()));
    }

    #[test]
    fn policy_identity_changes_without_changing_plan_identity() {
        let request = request(vec![binding(ROOT_A_HEX, "", "root")]);
        let first = prepare_composition(&request).unwrap();
        let mut changed = request.clone();
        changed.realization_policy.max_entries = changed.realization_policy.max_entries.saturating_sub(1);
        let second = prepare_composition(&changed).unwrap();

        assert_eq!(first.plan_ref, second.plan_ref);
        assert_ne!(first.realization_policy_ref, second.realization_policy_ref);
    }

    #[test]
    fn root_and_nested_mounts_merge_recursively() {
        let prepared =
            prepare_composition(&request(vec![binding(ROOT_A_HEX, "", "a"), binding(ROOT_B_HEX, "usr", "b")])).unwrap();
        let outcome = plan_composition(&prepared, vec![
            snapshot(ROOT_A_HEX, directory(&[("etc", directory(&[("a", file(FILE_A_HEX))]))])),
            snapshot(ROOT_B_HEX, directory(&[("bin", file(FILE_B_HEX))])),
        ])
        .unwrap();

        let SnapshotNode::Directory { entries } = outcome.root else {
            panic!("root must be a directory")
        };
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|entry| entry.name == "etc"));
        assert!(entries.iter().any(|entry| entry.name == "usr"));
    }

    #[test]
    fn identical_leaves_deduplicate() {
        let prepared =
            prepare_composition(&request(vec![binding(ROOT_A_HEX, "", "a"), binding(ROOT_B_HEX, "", "b")])).unwrap();
        let shared = directory(&[("same", file(FILE_A_HEX))]);
        let outcome =
            plan_composition(&prepared, vec![snapshot(ROOT_A_HEX, shared.clone()), snapshot(ROOT_B_HEX, shared)])
                .unwrap();

        assert!(outcome.merge_outcomes.iter().any(|item| item.kind == MergeOutcomeKind::IdenticalLeafDeduplicated));
        assert!(outcome.merge_outcomes.iter().any(|item| item.kind == MergeOutcomeKind::DirectoryUnion));
    }

    #[test]
    fn explicit_leaf_replacement_uses_derived_binding_ref() {
        let mut request = request(vec![binding(ROOT_A_HEX, "", "a"), binding(ROOT_B_HEX, "", "b")]);
        let preview = prepare_composition(&request).unwrap();
        let winner = preview
            .bindings
            .iter()
            .find(|item| item.root.digest_blake3 == ROOT_B_HEX)
            .unwrap()
            .binding_ref
            .clone();
        request.plan.collision_decisions.push(CollisionDecision {
            path: "bin/tool".to_string(),
            winner_binding_ref: winner.clone(),
        });
        let prepared = prepare_composition(&request).unwrap();
        let outcome = plan_composition(&prepared, vec![
            snapshot(ROOT_A_HEX, directory(&[("bin", directory(&[("tool", file(FILE_A_HEX))]))])),
            snapshot(ROOT_B_HEX, directory(&[("bin", directory(&[("tool", file(FILE_B_HEX))]))])),
        ])
        .unwrap();

        let replacement = outcome
            .merge_outcomes
            .iter()
            .find(|item| item.kind == MergeOutcomeKind::ExplicitReplacement)
            .unwrap();
        assert_eq!(replacement.path, "bin/tool");
        assert_eq!(replacement.winner_binding_ref.as_deref(), Some(winner.as_str()));
    }

    #[test]
    fn unsafe_paths_and_duplicate_bindings_fail() {
        let unsafe_request = request(vec![binding(ROOT_A_HEX, "../etc", "a")]);
        assert!(matches!(prepare_composition(&unsafe_request), Err(CompositionError::InvalidMountPath(_))));

        let duplicate = request(vec![binding(ROOT_A_HEX, "", "a"), binding(ROOT_A_HEX, "", "renamed")]);
        assert!(matches!(prepare_composition(&duplicate), Err(CompositionError::DuplicateBinding(_))));

        let empty = request(Vec::new());
        assert_eq!(prepare_composition(&empty).unwrap_err(), CompositionError::EmptyBindings);
    }

    #[test]
    fn missing_duplicate_stale_and_non_contributing_decisions_fail() {
        let base = request(vec![binding(ROOT_A_HEX, "", "a"), binding(ROOT_B_HEX, "", "b")]);
        let preview = prepare_composition(&base).unwrap();
        let winner = preview.bindings[0].binding_ref.clone();
        let roots = vec![
            snapshot(ROOT_A_HEX, directory(&[("tool", file(FILE_A_HEX))])),
            snapshot(ROOT_B_HEX, directory(&[("tool", file(FILE_B_HEX))])),
        ];
        assert!(matches!(plan_composition(&preview, roots.clone()), Err(CompositionError::UnresolvedCollision(_))));

        let mut duplicate = base.clone();
        duplicate.plan.collision_decisions = vec![
            CollisionDecision {
                path: "tool".to_string(),
                winner_binding_ref: winner.clone(),
            },
            CollisionDecision {
                path: "tool".to_string(),
                winner_binding_ref: winner.clone(),
            },
        ];
        assert!(matches!(prepare_composition(&duplicate), Err(CompositionError::DuplicateDecision(_))));

        let mut stale = base.clone();
        stale.plan.collision_decisions.push(CollisionDecision {
            path: "missing".to_string(),
            winner_binding_ref: winner,
        });
        let stale = prepare_composition(&stale).unwrap();
        let stale_roots = vec![
            snapshot(ROOT_A_HEX, directory(&[("left", file(FILE_A_HEX))])),
            snapshot(ROOT_B_HEX, directory(&[("right", file(FILE_B_HEX))])),
        ];
        assert!(matches!(plan_composition(&stale, stale_roots), Err(CompositionError::StaleDecision(_))));

        let mut outsider = base;
        outsider.plan.collision_decisions.push(CollisionDecision {
            path: "tool".to_string(),
            winner_binding_ref: format!("{BINDING_REF_PREFIX}{}", "f".repeat(BLAKE3_HEX_CHARS)),
        });
        let outsider = prepare_composition(&outsider).unwrap();
        assert!(matches!(plan_composition(&outsider, roots), Err(CompositionError::NonContributingWinner(_))));
    }

    #[test]
    fn snapshot_and_all_named_limits_fail_closed() {
        let mut invalid_policy = request(vec![binding(ROOT_A_HEX, "deep", "a")]);
        invalid_policy.realization_policy.max_bindings = 0;
        assert!(matches!(prepare_composition(&invalid_policy), Err(CompositionError::LimitInvalid("bindings"))));

        let mut binding_limit = request(vec![binding(ROOT_A_HEX, "deep", "a")]);
        binding_limit.realization_policy.max_bindings = 1;
        binding_limit.plan.bindings.push(binding(ROOT_B_HEX, "other", "b"));
        assert!(matches!(prepare_composition(&binding_limit), Err(CompositionError::LimitExceeded("bindings"))));

        let mut decision_limit = request(vec![binding(ROOT_A_HEX, "deep", "a")]);
        decision_limit.realization_policy.max_collision_decisions = 1;
        let winner = format!("{BINDING_REF_PREFIX}{}", "a".repeat(BLAKE3_HEX_CHARS));
        decision_limit.plan.collision_decisions = vec![
            CollisionDecision {
                path: "first".to_string(),
                winner_binding_ref: winner.clone(),
            },
            CollisionDecision {
                path: "second".to_string(),
                winner_binding_ref: winner,
            },
        ];
        assert!(matches!(
            prepare_composition(&decision_limit),
            Err(CompositionError::LimitExceeded("collision-decisions"))
        ));

        let mut path_limit = request(vec![binding(ROOT_A_HEX, "deep", "a")]);
        path_limit.realization_policy.max_path_bytes = 1;
        assert!(matches!(prepare_composition(&path_limit), Err(CompositionError::LimitExceeded("path-bytes"))));

        let mut entry_limit = request(vec![binding(ROOT_A_HEX, "", "a")]);
        entry_limit.realization_policy.max_entries = 1;
        let prepared = prepare_composition(&entry_limit).unwrap();
        let tree = directory(&[("a", file(FILE_A_HEX)), ("b", file(FILE_B_HEX))]);
        assert!(matches!(
            plan_composition(&prepared, vec![snapshot(ROOT_A_HEX, tree)]),
            Err(CompositionError::LimitExceeded("entries"))
        ));

        let mut depth_limit = request(vec![binding(ROOT_A_HEX, "", "a")]);
        depth_limit.realization_policy.max_depth = 1;
        let prepared = prepare_composition(&depth_limit).unwrap();
        let tree = directory(&[("a", directory(&[("b", file(FILE_A_HEX))]))]);
        assert!(matches!(
            plan_composition(&prepared, vec![snapshot(ROOT_A_HEX, tree)]),
            Err(CompositionError::LimitExceeded("depth"))
        ));

        let mut file_limit = request(vec![binding(ROOT_A_HEX, "", "a")]);
        file_limit.realization_policy.max_file_bytes = FILE_SIZE_BYTES.saturating_sub(1);
        let prepared = prepare_composition(&file_limit).unwrap();
        assert!(matches!(
            plan_composition(&prepared, vec![snapshot(ROOT_A_HEX, directory(&[("a", file(FILE_A_HEX))]))]),
            Err(CompositionError::LimitExceeded("file-bytes"))
        ));

        let mut total_limit = request(vec![binding(ROOT_A_HEX, "", "a")]);
        total_limit.realization_policy.max_total_file_bytes = FILE_SIZE_BYTES;
        let prepared = prepare_composition(&total_limit).unwrap();
        let tree = directory(&[("a", file(FILE_A_HEX)), ("b", file(FILE_B_HEX))]);
        assert!(matches!(
            plan_composition(&prepared, vec![snapshot(ROOT_A_HEX, tree)]),
            Err(CompositionError::LimitExceeded("total-file-bytes"))
        ));
    }

    #[test]
    fn missing_extra_and_non_directory_snapshots_fail() {
        let prepared = prepare_composition(&request(vec![binding(ROOT_A_HEX, "", "a")])).unwrap();
        assert!(matches!(plan_composition(&prepared, Vec::new()), Err(CompositionError::MissingSnapshot(_))));
        assert!(matches!(
            plan_composition(&prepared, vec![snapshot(ROOT_B_HEX, directory(&[]))]),
            Err(CompositionError::UnexpectedSnapshot(_))
        ));
        assert!(matches!(
            plan_composition(&prepared, vec![snapshot(ROOT_A_HEX, file(FILE_A_HEX))]),
            Err(CompositionError::RootNotDirectory(_))
        ));
    }

    #[test]
    fn symlink_policy_and_receipt_non_claims_are_explicit() {
        let mut symlink_request = request(vec![binding(ROOT_A_HEX, "", "a")]);
        symlink_request.realization_policy.allow_symlinks = false;
        let prepared = prepare_composition(&symlink_request).unwrap();
        let tree = directory(&[("link", SnapshotNode::Symlink {
            target: b"../target".to_vec(),
        })]);
        assert!(matches!(
            plan_composition(&prepared, vec![snapshot(ROOT_A_HEX, tree)]),
            Err(CompositionError::SymlinkRejected(_))
        ));

        let request = request(vec![binding(ROOT_A_HEX, "", "a")]);
        let prepared = prepare_composition(&request).unwrap();
        let outcome = plan_composition(&prepared, vec![snapshot(ROOT_A_HEX, directory(&[]))]).unwrap();
        let receipt = finalize_receipt(&prepared, &outcome, root(ROOT_A_HEX)).unwrap();
        assert_eq!(receipt.non_claim, NON_CLAIM);
        assert!(receipt.unsupported_metadata_classes.contains(&"ownership".to_string()));
    }
}
