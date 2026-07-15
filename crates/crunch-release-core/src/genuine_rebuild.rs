use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ReleaseEvidenceError;
use crate::manifest::validate_blake3_hex;
use crate::manifest::validation_error;

// r[impl mantle.build_correctness.release_determinism.identity_binding]
// r[impl mantle.build_correctness.release_determinism.authority_plan]
pub const CONTENT_BOUND_REBUILD_DESCRIPTOR_SCHEMA: &str = "mantle-content-bound-rebuild-descriptor-v1";
pub const REBUILD_AUTHORITY_PLAN_SCHEMA: &str = "mantle-rebuild-authority-plan-v1";
pub const MAX_REBUILD_ARGUMENT_COUNT: u32 = 128;
pub const MAX_REBUILD_INPUT_COUNT: u32 = 256;
pub const MAX_REBUILD_RUN_COUNT: u32 = 16;
pub const MAX_REBUILD_TEXT_BYTES: u32 = 16_384;
const MIN_GENUINE_REBUILD_RUN_COUNT: u32 = 2;
const ROOT_IDENTITIES_PER_RUN: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RebuildContentKind {
    RegularFile,
    Directory,
    SyntheticPolicy,
    Symlink,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RebuildInputRole {
    Source,
    Recipe,
    Executable,
    Tool,
    Provider,
    SandboxPolicy,
    EffectPolicy,
    NormalizationPolicy,
    PublishedTarget,
    WholeBundle,
    PriorOutput,
    OrdinaryOutput,
    Undeclared,
}

impl RebuildInputRole {
    fn is_declared_rebuild_input(self) -> bool {
        matches!(
            self,
            Self::Source
                | Self::Recipe
                | Self::Executable
                | Self::Tool
                | Self::Provider
                | Self::SandboxPolicy
                | Self::EffectPolicy
                | Self::NormalizationPolicy
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RebuildContentIdentity {
    pub name: String,
    pub role: RebuildInputRole,
    pub kind: RebuildContentKind,
    pub digest_blake3: String,
    pub size_bytes: u64,
}

impl RebuildContentIdentity {
    fn stable_key(&self) -> String {
        format!("{:?}:{}:{}", self.role, self.name, self.digest_blake3)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RebuildRunRootIdentity {
    pub run_id: String,
    pub output_root_identity: String,
    pub store_root_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebuildPolicyIdentities {
    pub sandbox_policy_blake3: String,
    pub effect_policy_blake3: String,
    pub normalization_policy_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentBoundRebuildDescriptor {
    pub schema: String,
    pub target_artifacts: Vec<RebuildContentIdentity>,
    pub recipe: RebuildContentIdentity,
    pub executable: RebuildContentIdentity,
    pub tools: Vec<RebuildContentIdentity>,
    pub ordered_arguments: Vec<String>,
    pub arguments_blake3: String,
    pub source_inputs: Vec<RebuildContentIdentity>,
    pub provider: RebuildContentIdentity,
    pub policies: RebuildPolicyIdentities,
    pub run_roots: Vec<RebuildRunRootIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebuildInputObservation {
    pub identity: RebuildContentIdentity,
    pub normalized_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesystem_object_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebuildRunRootObservation {
    pub identity: RebuildRunRootIdentity,
    pub normalized_output_path: String,
    pub normalized_store_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebuildAuthorityInput {
    pub descriptor: ContentBoundRebuildDescriptor,
    pub descriptor_blake3: String,
    pub published_targets: Vec<RebuildInputObservation>,
    pub candidate_inputs: Vec<RebuildInputObservation>,
    pub run_roots: Vec<RebuildRunRootObservation>,
    pub ordinary_output_path: String,
    pub proof_root_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RebuildAuthorityBlockerCode {
    DescriptorInvalid,
    DescriptorDigestMismatch,
    MissingDeclaredInput,
    UnexpectedDeclaredInput,
    TargetIdenticalInput,
    TargetObjectAlias,
    SymlinkInput,
    UnsupportedInputKind,
    WholeBundleInput,
    PublishedTargetInput,
    PriorOutputInput,
    OrdinaryOutputInput,
    UndeclaredInput,
    ReusedWriteRoot,
    OverlappingReadWriteRoot,
    OrdinaryOutputReuse,
    ProofRootReuse,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RebuildAuthorityBlocker {
    pub code: RebuildAuthorityBlockerCode,
    pub subject: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebuildAuthorityPlan {
    pub schema: String,
    pub descriptor_blake3: String,
    pub approved_read_identities: Vec<String>,
    pub approved_read_paths_blake3: String,
    pub fresh_write_root_identities: Vec<String>,
    pub target_authority_excluded: bool,
    pub blockers: Vec<RebuildAuthorityBlocker>,
}

struct TextField<'a> {
    value: &'a str,
    name: &'a str,
}

struct CandidateClassification<'a> {
    target_digests: &'a BTreeSet<String>,
    target_objects: &'a BTreeSet<String>,
    approved_read_identities: &'a mut Vec<String>,
    approved_read_paths: &'a mut Vec<String>,
    blockers: &'a mut Vec<RebuildAuthorityBlocker>,
}

#[derive(Clone, Copy)]
struct PathPair<'a> {
    candidate_path: &'a str,
    reference_path: &'a str,
}

impl RebuildAuthorityPlan {
    pub fn eligible(&self) -> bool {
        self.target_authority_excluded && self.blockers.is_empty()
    }
}

pub fn rebuild_arguments_digest_blake3(arguments: Vec<String>) -> Result<String, ReleaseEvidenceError> {
    validate_string_vec(&arguments, MAX_REBUILD_ARGUMENT_COUNT, "rebuild arguments")?;
    let bytes = serde_json::to_vec(&arguments)
        .map_err(|err| ReleaseEvidenceError::Parse(format!("serializing ordered rebuild arguments: {err}")))?;
    debug_assert!(!bytes.is_empty());
    debug_assert!(u32::try_from(arguments.len()).is_ok_and(|count| count <= MAX_REBUILD_ARGUMENT_COUNT));
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

pub fn canonical_content_bound_rebuild_descriptor(
    mut descriptor: ContentBoundRebuildDescriptor,
) -> Result<ContentBoundRebuildDescriptor, ReleaseEvidenceError> {
    validate_descriptor_header(&descriptor)?;
    descriptor.target_artifacts.sort();
    descriptor.tools.sort();
    descriptor.source_inputs.sort();
    descriptor.run_roots.sort_by(|left, right| left.run_id.cmp(&right.run_id));
    validate_descriptor_evidence(&descriptor)?;
    debug_assert_eq!(descriptor.schema, CONTENT_BOUND_REBUILD_DESCRIPTOR_SCHEMA);
    debug_assert!(!descriptor.target_artifacts.is_empty());
    Ok(descriptor)
}

pub fn content_bound_rebuild_descriptor_canonical_bytes(
    descriptor: ContentBoundRebuildDescriptor,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    let canonical = canonical_content_bound_rebuild_descriptor(descriptor)?;
    serde_json::to_vec(&canonical)
        .map_err(|err| ReleaseEvidenceError::Parse(format!("serializing content-bound rebuild descriptor: {err}")))
}

pub fn content_bound_rebuild_descriptor_digest_blake3(
    descriptor: ContentBoundRebuildDescriptor,
) -> Result<String, ReleaseEvidenceError> {
    let bytes = content_bound_rebuild_descriptor_canonical_bytes(descriptor)?;
    debug_assert!(!bytes.is_empty());
    debug_assert!(u32::try_from(bytes.len()).is_ok());
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

// r[impl mantle.build_correctness.release_determinism.genuine_rebuild]
// r[impl mantle.build_correctness.release_determinism.authority_plan]
pub fn plan_rebuild_authority(input: RebuildAuthorityInput) -> RebuildAuthorityPlan {
    let mut blockers = Vec::new();
    collect_descriptor_input_blockers(&input, &mut blockers);

    let target_digests = input
        .published_targets
        .iter()
        .map(|target| target.identity.digest_blake3.clone())
        .collect::<BTreeSet<_>>();
    let target_objects = input
        .published_targets
        .iter()
        .filter_map(|target| target.filesystem_object_identity.clone())
        .collect::<BTreeSet<_>>();
    let mut approved_read_identities = Vec::with_capacity(input.candidate_inputs.len());
    let mut approved_read_paths = Vec::with_capacity(input.candidate_inputs.len());
    {
        let mut classification = CandidateClassification {
            target_digests: &target_digests,
            target_objects: &target_objects,
            approved_read_identities: &mut approved_read_identities,
            approved_read_paths: &mut approved_read_paths,
            blockers: &mut blockers,
        };
        for candidate in &input.candidate_inputs {
            classify_candidate(candidate, &mut classification);
        }
    }
    classify_run_roots(&input, &approved_read_paths, &mut blockers);

    approved_read_identities.sort();
    approved_read_identities.dedup();
    approved_read_paths.sort();
    approved_read_paths.dedup();
    let approved_read_paths_blake3 = approved_read_paths_digest(&approved_read_paths, &mut blockers);
    let mut fresh_write_root_identities = input
        .run_roots
        .iter()
        .flat_map(|root| {
            [
                root.identity.output_root_identity.clone(),
                root.identity.store_root_identity.clone(),
            ]
        })
        .collect::<Vec<_>>();
    fresh_write_root_identities.sort();
    fresh_write_root_identities.dedup();
    blockers.sort();
    blockers.dedup();
    let is_target_authority_excluded = !blockers.iter().any(is_target_authority_blocker);
    let plan = RebuildAuthorityPlan {
        schema: REBUILD_AUTHORITY_PLAN_SCHEMA.to_string(),
        descriptor_blake3: input.descriptor_blake3,
        approved_read_identities,
        approved_read_paths_blake3,
        fresh_write_root_identities,
        target_authority_excluded: is_target_authority_excluded,
        blockers,
    };
    debug_assert_eq!(plan.schema, REBUILD_AUTHORITY_PLAN_SCHEMA);
    debug_assert_eq!(plan.eligible(), plan.target_authority_excluded && plan.blockers.is_empty());
    plan
}

fn approved_read_paths_digest(approved_read_paths: &[String], blockers: &mut Vec<RebuildAuthorityBlocker>) -> String {
    debug_assert!(approved_read_paths.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert!(!REBUILD_AUTHORITY_PLAN_SCHEMA.is_empty());
    match digest_string_list(approved_read_paths) {
        Ok(digest_blake3) => digest_blake3,
        Err(error) => {
            push_blocker(
                blockers,
                RebuildAuthorityBlockerCode::DescriptorInvalid,
                "approved-read-paths",
                error.to_string(),
            );
            String::new()
        }
    }
}

pub fn rebuild_authority_plan_canonical_bytes(mut plan: RebuildAuthorityPlan) -> Result<Vec<u8>, ReleaseEvidenceError> {
    if plan.schema != REBUILD_AUTHORITY_PLAN_SCHEMA {
        return Err(validation_error(format!(
            "rebuild authority plan schema must be {REBUILD_AUTHORITY_PLAN_SCHEMA}, got {}",
            plan.schema
        )));
    }
    validate_blake3_hex(&plan.descriptor_blake3, "rebuild authority plan descriptor_blake3")?;
    validate_blake3_hex(&plan.approved_read_paths_blake3, "rebuild authority plan approved_read_paths_blake3")?;
    validate_string_vec(&plan.approved_read_identities, MAX_REBUILD_INPUT_COUNT, "approved read identities")?;
    validate_string_vec(
        &plan.fresh_write_root_identities,
        MAX_REBUILD_RUN_COUNT.saturating_mul(ROOT_IDENTITIES_PER_RUN),
        "write roots",
    )?;
    plan.approved_read_identities.sort();
    plan.approved_read_identities.dedup();
    plan.fresh_write_root_identities.sort();
    plan.fresh_write_root_identities.dedup();
    plan.blockers.sort();
    plan.blockers.dedup();
    debug_assert!(plan.approved_read_identities.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert!(plan.fresh_write_root_identities.windows(2).all(|pair| pair[0] < pair[1]));
    serde_json::to_vec(&plan)
        .map_err(|err| ReleaseEvidenceError::Parse(format!("serializing rebuild authority plan: {err}")))
}

pub fn rebuild_authority_plan_digest_blake3(plan: RebuildAuthorityPlan) -> Result<String, ReleaseEvidenceError> {
    let bytes = rebuild_authority_plan_canonical_bytes(plan)?;
    debug_assert!(!bytes.is_empty());
    debug_assert!(u32::try_from(bytes.len()).is_ok());
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn collect_descriptor_input_blockers(input: &RebuildAuthorityInput, blockers: &mut Vec<RebuildAuthorityBlocker>) {
    let blocker_count_before = blockers.len();
    let canonical_descriptor = canonical_content_bound_rebuild_descriptor(input.descriptor.clone());
    let expected_descriptor_blake3 =
        canonical_descriptor.clone().and_then(content_bound_rebuild_descriptor_digest_blake3);
    if let Err(error) = &canonical_descriptor {
        push_blocker(blockers, RebuildAuthorityBlockerCode::DescriptorInvalid, "descriptor", error.to_string());
    }
    if let Ok(expected) = expected_descriptor_blake3
        && expected != input.descriptor_blake3
    {
        push_blocker(
            blockers,
            RebuildAuthorityBlockerCode::DescriptorDigestMismatch,
            "descriptor",
            format!("expected {expected}, observed {}", input.descriptor_blake3),
        );
    }

    let declared = canonical_descriptor.as_ref().map(descriptor_input_keys).unwrap_or_default();
    let observed = input.candidate_inputs.iter().map(|item| item.identity.stable_key()).collect::<BTreeSet<_>>();
    for missing in declared.difference(&observed) {
        push_blocker(
            blockers,
            RebuildAuthorityBlockerCode::MissingDeclaredInput,
            missing,
            "descriptor input was not observed",
        );
    }
    for unexpected in observed.difference(&declared) {
        push_blocker(
            blockers,
            RebuildAuthorityBlockerCode::UnexpectedDeclaredInput,
            unexpected,
            "observed input is absent from descriptor",
        );
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(canonical_descriptor.is_ok() || blockers.len() > blocker_count_before);
}

fn validate_descriptor_header(descriptor: &ContentBoundRebuildDescriptor) -> Result<(), ReleaseEvidenceError> {
    if descriptor.schema != CONTENT_BOUND_REBUILD_DESCRIPTOR_SCHEMA {
        return Err(validation_error(format!(
            "content-bound rebuild descriptor schema must be {CONTENT_BOUND_REBUILD_DESCRIPTOR_SCHEMA}, got {}",
            descriptor.schema
        )));
    }
    validate_blake3_hex(&descriptor.arguments_blake3, "rebuild descriptor arguments_blake3")?;
    validate_blake3_hex(&descriptor.policies.sandbox_policy_blake3, "rebuild descriptor sandbox policy")?;
    validate_blake3_hex(&descriptor.policies.effect_policy_blake3, "rebuild descriptor effect policy")?;
    validate_blake3_hex(&descriptor.policies.normalization_policy_blake3, "rebuild descriptor normalization policy")?;
    Ok(())
}

fn validate_descriptor_evidence(descriptor: &ContentBoundRebuildDescriptor) -> Result<(), ReleaseEvidenceError> {
    debug_assert!(MAX_REBUILD_INPUT_COUNT > 0);
    debug_assert!(MAX_REBUILD_RUN_COUNT >= MIN_GENUINE_REBUILD_RUN_COUNT);
    validate_identity_vec(&descriptor.target_artifacts, MAX_REBUILD_INPUT_COUNT, "target artifacts")?;
    validate_identity(&descriptor.recipe, RebuildInputRole::Recipe, "recipe")?;
    validate_identity(&descriptor.executable, RebuildInputRole::Executable, "executable")?;
    validate_identity_vec(&descriptor.tools, MAX_REBUILD_INPUT_COUNT, "tools")?;
    validate_identity_vec(&descriptor.source_inputs, MAX_REBUILD_INPUT_COUNT, "source inputs")?;
    validate_identity(&descriptor.provider, RebuildInputRole::Provider, "provider")?;
    validate_string_vec(&descriptor.ordered_arguments, MAX_REBUILD_ARGUMENT_COUNT, "ordered arguments")?;
    let arguments_blake3 = rebuild_arguments_digest_blake3(descriptor.ordered_arguments.clone())?;
    if arguments_blake3 != descriptor.arguments_blake3 {
        return Err(validation_error("content-bound rebuild descriptor ordered arguments digest mismatch".to_string()));
    }
    if descriptor.target_artifacts.is_empty() {
        return Err(validation_error(
            "content-bound rebuild descriptor target_artifacts must not be empty".to_string(),
        ));
    }
    if descriptor.source_inputs.is_empty() {
        return Err(validation_error("content-bound rebuild descriptor source_inputs must not be empty".to_string()));
    }
    for target in &descriptor.target_artifacts {
        validate_identity(target, RebuildInputRole::PublishedTarget, "target artifact")?;
    }
    for tool in &descriptor.tools {
        validate_identity(tool, RebuildInputRole::Tool, "tool")?;
    }
    for source in &descriptor.source_inputs {
        validate_identity(source, RebuildInputRole::Source, "source input")?;
    }
    validate_run_roots(&descriptor.run_roots)?;
    debug_assert!(!descriptor.target_artifacts.is_empty());
    debug_assert!(!descriptor.source_inputs.is_empty());
    Ok(())
}

fn validate_identity_vec(
    identities: &[RebuildContentIdentity],
    max_count: u32,
    field: &str,
) -> Result<(), ReleaseEvidenceError> {
    bounded_count(identities.len(), max_count, field)?;
    let mut seen = BTreeSet::new();
    for identity in identities {
        validate_identity_shape(identity, field)?;
        if !seen.insert(identity.stable_key()) {
            return Err(validation_error(format!(
                "content-bound rebuild descriptor {field} contains duplicate identity"
            )));
        }
    }
    Ok(())
}

fn validate_identity(
    identity: &RebuildContentIdentity,
    expected_role: RebuildInputRole,
    field: &str,
) -> Result<(), ReleaseEvidenceError> {
    validate_identity_shape(identity, field)?;
    if identity.role != expected_role {
        return Err(validation_error(format!(
            "content-bound rebuild descriptor {field} role must be {:?}, got {:?}",
            expected_role, identity.role
        )));
    }
    Ok(())
}

fn validate_identity_shape(identity: &RebuildContentIdentity, field: &str) -> Result<(), ReleaseEvidenceError> {
    let field_name = format!("{field}.name");
    validate_text(TextField {
        value: &identity.name,
        name: &field_name,
    })?;
    validate_blake3_hex(&identity.digest_blake3, &format!("content-bound rebuild descriptor {field} digest"))?;
    if identity.kind == RebuildContentKind::Symlink || identity.kind == RebuildContentKind::Other {
        return Err(validation_error(format!(
            "content-bound rebuild descriptor {field} uses unsupported content kind {:?}",
            identity.kind
        )));
    }
    Ok(())
}

fn validate_run_roots(run_roots: &[RebuildRunRootIdentity]) -> Result<(), ReleaseEvidenceError> {
    bounded_count(run_roots.len(), MAX_REBUILD_RUN_COUNT, "run roots")?;
    let run_count = u32::try_from(run_roots.len())
        .map_err(|_| validation_error("content-bound rebuild descriptor run count overflowed u32".to_string()))?;
    if run_count < MIN_GENUINE_REBUILD_RUN_COUNT {
        return Err(validation_error(format!(
            "content-bound rebuild descriptor requires at least {MIN_GENUINE_REBUILD_RUN_COUNT} run roots"
        )));
    }
    let mut ids = BTreeSet::new();
    let mut roots = BTreeSet::new();
    for root in run_roots {
        validate_text(TextField {
            value: &root.run_id,
            name: "run root id",
        })?;
        validate_text(TextField {
            value: &root.output_root_identity,
            name: "output root identity",
        })?;
        validate_text(TextField {
            value: &root.store_root_identity,
            name: "store root identity",
        })?;
        if !ids.insert(root.run_id.clone()) {
            return Err(validation_error(format!("duplicate rebuild run root id {}", root.run_id)));
        }
        if !roots.insert(root.output_root_identity.clone()) || !roots.insert(root.store_root_identity.clone()) {
            return Err(validation_error("rebuild run roots must be distinct".to_string()));
        }
    }
    debug_assert_eq!(ids.len(), run_roots.len());
    debug_assert_eq!(roots.len(), run_roots.len().saturating_add(run_roots.len()));
    Ok(())
}

fn validate_string_vec(values: &[String], max_count: u32, field: &str) -> Result<(), ReleaseEvidenceError> {
    bounded_count(values.len(), max_count, field)?;
    for value in values {
        validate_text(TextField { value, name: field })?;
    }
    Ok(())
}

fn validate_text(field: TextField<'_>) -> Result<(), ReleaseEvidenceError> {
    if field.value.trim().is_empty() {
        return Err(validation_error(format!("{} must not be empty", field.name)));
    }
    let bytes_count = u32::try_from(field.value.len())
        .map_err(|_| validation_error(format!("{} byte length overflowed u32", field.name)))?;
    if bytes_count > MAX_REBUILD_TEXT_BYTES {
        return Err(validation_error(format!("{} exceeds {MAX_REBUILD_TEXT_BYTES} UTF-8 bytes", field.name)));
    }
    Ok(())
}

fn bounded_count(actual: usize, maximum: u32, field: &str) -> Result<(), ReleaseEvidenceError> {
    let actual = u32::try_from(actual).map_err(|_| validation_error(format!("{field} count overflowed u32")))?;
    if actual > maximum {
        return Err(validation_error(format!("{field} count {actual} exceeds {maximum}")));
    }
    Ok(())
}

fn descriptor_input_keys(descriptor: &ContentBoundRebuildDescriptor) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    keys.insert(descriptor.recipe.stable_key());
    keys.insert(descriptor.executable.stable_key());
    keys.insert(descriptor.provider.stable_key());
    keys.extend(descriptor.tools.iter().map(RebuildContentIdentity::stable_key));
    keys.extend(descriptor.source_inputs.iter().map(RebuildContentIdentity::stable_key));
    keys.insert(policy_key(RebuildInputRole::SandboxPolicy, &descriptor.policies.sandbox_policy_blake3));
    keys.insert(policy_key(RebuildInputRole::EffectPolicy, &descriptor.policies.effect_policy_blake3));
    keys.insert(policy_key(RebuildInputRole::NormalizationPolicy, &descriptor.policies.normalization_policy_blake3));
    keys
}

fn policy_key(role: RebuildInputRole, digest: &str) -> String {
    format!("{:?}:policy:{digest}", role)
}

fn classify_candidate(candidate: &RebuildInputObservation, classification: &mut CandidateClassification<'_>) {
    let subject = candidate.identity.name.as_str();
    let blocker_count_before = classification.blockers.len();
    debug_assert!(MAX_REBUILD_INPUT_COUNT > 0);
    debug_assert!(!REBUILD_AUTHORITY_PLAN_SCHEMA.is_empty());
    if !candidate.identity.role.is_declared_rebuild_input() {
        let code = role_blocker_code(candidate.identity.role);
        push_blocker(classification.blockers, code, subject, "input role cannot receive rebuild read authority");
    }
    match candidate.identity.kind {
        RebuildContentKind::Symlink => push_blocker(
            classification.blockers,
            RebuildAuthorityBlockerCode::SymlinkInput,
            subject,
            "symlink roots are not rebuild authority",
        ),
        RebuildContentKind::Other => push_blocker(
            classification.blockers,
            RebuildAuthorityBlockerCode::UnsupportedInputKind,
            subject,
            "unsupported filesystem object cannot be rebuild authority",
        ),
        RebuildContentKind::RegularFile | RebuildContentKind::Directory | RebuildContentKind::SyntheticPolicy => {}
    }
    if classification.target_digests.contains(&candidate.identity.digest_blake3) {
        push_blocker(
            classification.blockers,
            RebuildAuthorityBlockerCode::TargetIdenticalInput,
            subject,
            "input BLAKE3 equals a published target",
        );
    }
    match &candidate.filesystem_object_identity {
        Some(object) if classification.target_objects.contains(object) => push_blocker(
            classification.blockers,
            RebuildAuthorityBlockerCode::TargetObjectAlias,
            subject,
            "input filesystem object aliases a published target",
        ),
        Some(_) | None => {}
    }
    let is_candidate_blocked =
        classification.blockers[blocker_count_before..].iter().any(|blocker| blocker.subject == subject);
    if !is_candidate_blocked {
        classification.approved_read_identities.push(candidate.identity.stable_key());
        classification.approved_read_paths.push(candidate.normalized_path.clone());
    }
    debug_assert!(
        is_candidate_blocked || classification.approved_read_paths.last() == Some(&candidate.normalized_path)
    );
    debug_assert!(classification.blockers.len() >= blocker_count_before);
}

fn role_blocker_code(role: RebuildInputRole) -> RebuildAuthorityBlockerCode {
    match role {
        RebuildInputRole::WholeBundle => RebuildAuthorityBlockerCode::WholeBundleInput,
        RebuildInputRole::PublishedTarget => RebuildAuthorityBlockerCode::PublishedTargetInput,
        RebuildInputRole::PriorOutput => RebuildAuthorityBlockerCode::PriorOutputInput,
        RebuildInputRole::OrdinaryOutput => RebuildAuthorityBlockerCode::OrdinaryOutputInput,
        _ => RebuildAuthorityBlockerCode::UndeclaredInput,
    }
}

fn classify_run_roots(
    input: &RebuildAuthorityInput,
    approved_read_paths: &[String],
    blockers: &mut Vec<RebuildAuthorityBlocker>,
) {
    let blocker_count_before = blockers.len();
    let mut write_paths = BTreeSet::new();
    for root in &input.run_roots {
        for (identity, path) in [
            (&root.identity.output_root_identity, &root.normalized_output_path),
            (&root.identity.store_root_identity, &root.normalized_store_path),
        ] {
            if !write_paths.insert(path.clone()) {
                push_blocker(
                    blockers,
                    RebuildAuthorityBlockerCode::ReusedWriteRoot,
                    identity,
                    format!("write path {path} is reused"),
                );
            }
            if paths_overlap(PathPair {
                candidate_path: path,
                reference_path: &input.ordinary_output_path,
            }) {
                push_blocker(
                    blockers,
                    RebuildAuthorityBlockerCode::OrdinaryOutputReuse,
                    identity,
                    "write root overlaps ordinary rebuild output",
                );
            }
            if path == &input.proof_root_path {
                push_blocker(
                    blockers,
                    RebuildAuthorityBlockerCode::ProofRootReuse,
                    identity,
                    "write root cannot equal proof root",
                );
            }
            for read_path in approved_read_paths {
                if paths_overlap(PathPair {
                    candidate_path: path,
                    reference_path: read_path,
                }) {
                    push_blocker(
                        blockers,
                        RebuildAuthorityBlockerCode::OverlappingReadWriteRoot,
                        identity,
                        format!("write root overlaps approved read path {read_path}"),
                    );
                }
            }
        }
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(write_paths.len() <= input.run_roots.len().saturating_add(input.run_roots.len()));
}

fn paths_overlap(paths: PathPair<'_>) -> bool {
    paths.candidate_path == paths.reference_path
        || path_is_descendant(paths)
        || path_is_descendant(PathPair {
            candidate_path: paths.reference_path,
            reference_path: paths.candidate_path,
        })
}

fn path_is_descendant(paths: PathPair<'_>) -> bool {
    if paths.reference_path == "/" {
        return paths.candidate_path.starts_with('/') && paths.candidate_path != paths.reference_path;
    }
    let prefix = format!("{}/", paths.reference_path.trim_end_matches('/'));
    paths.candidate_path.starts_with(&prefix)
}

fn push_blocker(
    blockers: &mut Vec<RebuildAuthorityBlocker>,
    code: RebuildAuthorityBlockerCode,
    subject: impl Into<String>,
    detail: impl Into<String>,
) {
    blockers.push(RebuildAuthorityBlocker {
        code,
        subject: subject.into(),
        detail: detail.into(),
    });
}

fn is_target_authority_blocker(blocker: &RebuildAuthorityBlocker) -> bool {
    matches!(
        blocker.code,
        RebuildAuthorityBlockerCode::TargetIdenticalInput
            | RebuildAuthorityBlockerCode::TargetObjectAlias
            | RebuildAuthorityBlockerCode::SymlinkInput
            | RebuildAuthorityBlockerCode::WholeBundleInput
            | RebuildAuthorityBlockerCode::PublishedTargetInput
            | RebuildAuthorityBlockerCode::PriorOutputInput
            | RebuildAuthorityBlockerCode::OrdinaryOutputInput
            | RebuildAuthorityBlockerCode::OrdinaryOutputReuse
    )
}

fn digest_string_list(values: &[String]) -> Result<String, ReleaseEvidenceError> {
    let bytes = serde_json::to_vec(values)
        .map_err(|error| ReleaseEvidenceError::Parse(format!("serializing rebuild authority path list: {error}")))?;
    debug_assert!(!bytes.is_empty());
    debug_assert!(bytes.starts_with(b"["));
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::manifest::BLAKE3_HEX_LENGTH_CHARS;

    fn digest(seed: u8) -> String {
        format!("{:x}", seed % 16).repeat(BLAKE3_HEX_LENGTH_CHARS)
    }

    fn identity(name: &str, role: RebuildInputRole, seed: u8) -> RebuildContentIdentity {
        RebuildContentIdentity {
            name: name.to_string(),
            role,
            kind: RebuildContentKind::RegularFile,
            digest_blake3: digest(seed),
            size_bytes: u64::from(seed).saturating_add(1),
        }
    }

    fn policy_identity(name: &str, role: RebuildInputRole, seed: u8) -> RebuildInputObservation {
        RebuildInputObservation {
            identity: RebuildContentIdentity {
                name: name.to_string(),
                role,
                kind: RebuildContentKind::SyntheticPolicy,
                digest_blake3: digest(seed),
                size_bytes: 1,
            },
            normalized_path: format!("policy:{name}"),
            filesystem_object_identity: None,
        }
    }

    fn run_root(run_id: &str) -> RebuildRunRootIdentity {
        RebuildRunRootIdentity {
            run_id: run_id.to_string(),
            output_root_identity: format!("{run_id}:output"),
            store_root_identity: format!("{run_id}:store"),
        }
    }

    fn descriptor() -> ContentBoundRebuildDescriptor {
        let arguments = vec!["sh".to_string(), "input:recipe".to_string()];
        ContentBoundRebuildDescriptor {
            schema: CONTENT_BOUND_REBUILD_DESCRIPTOR_SCHEMA.to_string(),
            target_artifacts: vec![identity("bin/mantle", RebuildInputRole::PublishedTarget, 1)],
            recipe: identity("release-rebuild-recipe", RebuildInputRole::Recipe, 2),
            executable: identity("busybox", RebuildInputRole::Executable, 3),
            tools: vec![identity("compiler", RebuildInputRole::Tool, 4)],
            ordered_arguments: arguments.clone(),
            arguments_blake3: rebuild_arguments_digest_blake3(arguments).unwrap(),
            source_inputs: vec![identity("source-archive", RebuildInputRole::Source, 5)],
            provider: identity("provider-inventory", RebuildInputRole::Provider, 6),
            policies: RebuildPolicyIdentities {
                sandbox_policy_blake3: digest(7),
                effect_policy_blake3: digest(8),
                normalization_policy_blake3: digest(9),
            },
            run_roots: vec![run_root("run-000"), run_root("run-001")],
        }
    }

    fn observation(identity: RebuildContentIdentity, path: &str) -> RebuildInputObservation {
        RebuildInputObservation {
            identity,
            normalized_path: path.to_string(),
            filesystem_object_identity: None,
        }
    }

    fn authority_input() -> RebuildAuthorityInput {
        let descriptor = descriptor();
        let descriptor_blake3 = content_bound_rebuild_descriptor_digest_blake3(descriptor.clone()).unwrap();
        let mut candidate_inputs = vec![
            observation(descriptor.recipe.clone(), "/inputs/recipe"),
            observation(descriptor.executable.clone(), "/inputs/executable"),
            observation(descriptor.tools[0].clone(), "/inputs/compiler"),
            observation(descriptor.source_inputs[0].clone(), "/inputs/source.tar"),
            observation(descriptor.provider.clone(), "/inputs/provider.json"),
            policy_identity("policy", RebuildInputRole::SandboxPolicy, 7),
            policy_identity("policy", RebuildInputRole::EffectPolicy, 8),
            policy_identity("policy", RebuildInputRole::NormalizationPolicy, 9),
        ];
        candidate_inputs.sort_by(|left, right| left.identity.name.cmp(&right.identity.name));
        RebuildAuthorityInput {
            published_targets: vec![observation(
                descriptor.target_artifacts[0].clone(),
                "/bundle/bin/mantle",
            )],
            candidate_inputs,
            run_roots: vec![
                RebuildRunRootObservation {
                    identity: descriptor.run_roots[0].clone(),
                    normalized_output_path: "/proof/run-000/output".to_string(),
                    normalized_store_path: "/proof/run-000/store".to_string(),
                },
                RebuildRunRootObservation {
                    identity: descriptor.run_roots[1].clone(),
                    normalized_output_path: "/proof/run-001/output".to_string(),
                    normalized_store_path: "/proof/run-001/store".to_string(),
                },
            ],
            descriptor,
            descriptor_blake3,
            ordinary_output_path: "/ordinary-output".to_string(),
            proof_root_path: "/proof".to_string(),
        }
    }

    // r[verify mantle.build_correctness.release_determinism.authority_plan.test]
    #[test]
    fn authority_plan_accepts_declared_content_bound_inputs() {
        let plan = plan_rebuild_authority(authority_input());

        assert!(plan.eligible(), "{:?}", plan.blockers);
        assert!(plan.target_authority_excluded);
        assert_eq!(plan.approved_read_identities.len(), 8);
        assert_eq!(plan.fresh_write_root_identities.len(), 4);
    }

    #[test]
    fn authority_plan_blocks_target_copy_alias_and_symlink() {
        for (role, kind, object_identity, expected) in [
            (
                RebuildInputRole::Tool,
                RebuildContentKind::RegularFile,
                None,
                RebuildAuthorityBlockerCode::TargetIdenticalInput,
            ),
            (
                RebuildInputRole::Tool,
                RebuildContentKind::RegularFile,
                Some("device=1;inode=2".to_string()),
                RebuildAuthorityBlockerCode::TargetObjectAlias,
            ),
            (RebuildInputRole::Tool, RebuildContentKind::Symlink, None, RebuildAuthorityBlockerCode::SymlinkInput),
        ] {
            let mut input = authority_input();
            let mut attack = identity("target-alias", role, 1);
            attack.kind = kind;
            input.candidate_inputs.push(RebuildInputObservation {
                identity: attack,
                normalized_path: "/inputs/target-alias".to_string(),
                filesystem_object_identity: object_identity.clone(),
            });
            if object_identity.is_some() {
                input.published_targets[0].filesystem_object_identity = object_identity;
            }
            let plan = plan_rebuild_authority(input);
            assert!(!plan.eligible());
            assert!(plan.blockers.iter().any(|blocker| blocker.code == expected), "{:?}", plan.blockers);
        }
    }

    #[test]
    fn authority_plan_blocks_whole_bundle_prior_and_ordinary_outputs() {
        for (role, expected) in [
            (RebuildInputRole::WholeBundle, RebuildAuthorityBlockerCode::WholeBundleInput),
            (RebuildInputRole::PriorOutput, RebuildAuthorityBlockerCode::PriorOutputInput),
            (RebuildInputRole::OrdinaryOutput, RebuildAuthorityBlockerCode::OrdinaryOutputInput),
        ] {
            let mut input = authority_input();
            input.candidate_inputs.push(observation(identity("forbidden", role, 10), "/forbidden"));
            let plan = plan_rebuild_authority(input);
            assert!(!plan.eligible());
            assert!(plan.blockers.iter().any(|blocker| blocker.code == expected), "{:?}", plan.blockers);
        }
    }

    #[test]
    fn authority_plan_blocks_reused_and_overlapping_roots() {
        let mut reused = authority_input();
        reused.run_roots[1].normalized_store_path = reused.run_roots[0].normalized_store_path.clone();
        let reused_plan = plan_rebuild_authority(reused);
        assert!(!reused_plan.eligible());
        assert!(
            reused_plan
                .blockers
                .iter()
                .any(|blocker| blocker.code == RebuildAuthorityBlockerCode::ReusedWriteRoot)
        );

        let mut overlapping = authority_input();
        overlapping.run_roots[0].normalized_output_path = "/inputs/recipe/nested".to_string();
        let overlapping_plan = plan_rebuild_authority(overlapping);
        assert!(!overlapping_plan.eligible());
        assert!(
            overlapping_plan
                .blockers
                .iter()
                .any(|blocker| blocker.code == RebuildAuthorityBlockerCode::OverlappingReadWriteRoot)
        );
    }

    // r[verify mantle.build_correctness.release_determinism.fixtures.negative.identity_drift]
    #[test]
    fn descriptor_digest_changes_for_every_authority_identity_drift() {
        const DRIFT_VARIANT_COUNT: u32 = 9;
        let base = descriptor();
        let base_digest = content_bound_rebuild_descriptor_digest_blake3(base.clone()).unwrap();
        let mut variants = Vec::new();

        let mut recipe = base.clone();
        recipe.recipe.digest_blake3 = digest(10);
        variants.push(recipe);
        let mut executable = base.clone();
        executable.executable.digest_blake3 = digest(11);
        variants.push(executable);
        let mut tool = base.clone();
        tool.tools[0].digest_blake3 = digest(12);
        variants.push(tool);
        let mut source = base.clone();
        source.source_inputs[0].digest_blake3 = digest(13);
        variants.push(source);
        let mut arguments = base.clone();
        arguments.ordered_arguments.push("--drift".to_string());
        arguments.arguments_blake3 = rebuild_arguments_digest_blake3(arguments.ordered_arguments.clone()).unwrap();
        variants.push(arguments);
        let mut provider = base.clone();
        provider.provider.digest_blake3 = digest(14);
        variants.push(provider);
        let mut sandbox_policy = base.clone();
        sandbox_policy.policies.sandbox_policy_blake3 = digest(15);
        variants.push(sandbox_policy);
        let mut effect_policy = base.clone();
        effect_policy.policies.effect_policy_blake3 = digest(16);
        variants.push(effect_policy);
        let mut normalization_policy = base.clone();
        normalization_policy.policies.normalization_policy_blake3 = digest(17);
        variants.push(normalization_policy);

        assert_eq!(u32::try_from(variants.len()).unwrap(), DRIFT_VARIANT_COUNT);
        for variant in &variants {
            assert_eq!(variant.recipe.name, base.recipe.name);
            assert_eq!(variant.executable.name, base.executable.name);
            assert_eq!(variant.tools[0].name, base.tools[0].name);
            assert_eq!(variant.source_inputs[0].name, base.source_inputs[0].name);
            assert_eq!(variant.provider.name, base.provider.name);
        }
        for variant in variants {
            let drifted = content_bound_rebuild_descriptor_digest_blake3(variant).unwrap();
            assert_ne!(drifted, base_digest);
        }
    }

    #[test]
    fn descriptor_rejects_stale_argument_digest_and_single_run() {
        let mut stale = descriptor();
        stale.ordered_arguments.push("--changed".to_string());
        let stale_error = canonical_content_bound_rebuild_descriptor(stale).unwrap_err();
        assert!(stale_error.to_string().contains("arguments digest mismatch"));

        let mut one_run = descriptor();
        one_run.run_roots.pop();
        let run_error = canonical_content_bound_rebuild_descriptor(one_run).unwrap_err();
        assert!(run_error.to_string().contains("at least 2 run roots"));
    }
}
