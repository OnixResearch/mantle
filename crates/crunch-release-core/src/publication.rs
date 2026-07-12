use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Serialize;

use crate::manifest::BLAKE3_HEX_LENGTH_CHARS;
use crate::manifest::BundledArtifactKind;

pub const RELEASE_PUBLICATION_PLAN_SCHEMA: &str = "mantle-release-publication-plan-v1";
pub const RELEASE_PUBLICATION_ARTIFACTS_COUNT_MAX: u32 = 256;
pub const RELEASE_PUBLICATION_POLICY_FACTS_COUNT_MAX: u32 = 256;
pub const RELEASE_PUBLICATION_FAILURES_COUNT_MAX: u32 = 2;
pub const RELEASE_PUBLICATION_TEXT_BYTES_MAX: u32 = 4_096;
pub const RELEASE_PUBLICATION_PATH_COMPONENTS_MAX: u32 = 128;

const MANIFEST_RELATIVE_PATH: &str = "manifest.json";
const PATH_SEPARATOR: char = '/';
const WINDOWS_PATH_SEPARATOR: char = '\\';
const NUL_CHARACTER: char = '\0';
const WINDOW_PAIR_COUNT: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PublicationDestinationObservation {
    Absent,
    File,
    Symlink,
    EmptyDirectory,
    NonEmptyDirectory,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublicationArtifactInput {
    pub relative_path: String,
    pub kind: BundledArtifactKind,
    pub size_bytes: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublicationPolicyFact {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationPlanRequest {
    pub release_id: String,
    pub destination: PublicationDestinationObservation,
    pub artifacts: Vec<PublicationArtifactInput>,
    pub policy: Vec<PublicationPolicyFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublicationPlan {
    pub schema: String,
    pub release_id: String,
    pub artifacts: Vec<PublicationArtifactInput>,
    pub policy: Vec<PublicationPolicyFact>,
    pub plan_identity_blake3: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PublicationPlanBlockerKind {
    DestinationExists,
    EmptyReleaseId,
    ReleaseIdTooLong,
    EmptyArtifacts,
    ArtifactCountOverflow,
    ArtifactCountExceeded,
    EmptyPolicy,
    PolicyCountOverflow,
    PolicyCountExceeded,
    EmptyPolicyName,
    PolicyNameTooLong,
    PolicyValueTooLong,
    DuplicatePolicyName,
    InvalidArtifactPath,
    ArtifactPathTooLong,
    ArtifactPathTooDeep,
    ManifestPathReserved,
    DuplicateArtifactPath,
    OverlappingArtifactPath,
    InvalidArtifactDigest,
    StagedArtifactMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationPlanBlocker {
    pub kind: PublicationPlanBlockerKind,
    pub subject: Option<String>,
    pub message: String,
}

// r[impl mantle.release_provenance.bundle_publication.boundary]
pub fn plan_release_publication(
    mut request: PublicationPlanRequest,
) -> Result<PublicationPlan, Vec<PublicationPlanBlocker>> {
    let mut blockers = validate_plan_request(&request);
    request.artifacts.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    request
        .policy
        .sort_by(|left, right| left.name.cmp(&right.name).then_with(|| left.value.cmp(&right.value)));
    blockers.extend(validate_sorted_artifact_relationships(&request.artifacts));
    blockers.extend(validate_sorted_policy_relationships(&request.policy));
    sort_blockers(&mut blockers);
    if !blockers.is_empty() {
        return Err(blockers);
    }

    let plan_identity_blake3 = publication_plan_identity(&request.release_id, &request.artifacts, &request.policy);
    assert!(!request.artifacts.is_empty(), "valid publication plan must include artifacts");
    assert!(!request.policy.is_empty(), "valid publication plan must include policy facts");
    assert_eq!(plan_identity_blake3.len(), BLAKE3_HEX_LENGTH_CHARS);
    Ok(PublicationPlan {
        schema: RELEASE_PUBLICATION_PLAN_SCHEMA.to_string(),
        release_id: request.release_id,
        artifacts: request.artifacts,
        policy: request.policy,
        plan_identity_blake3,
    })
}

fn validate_plan_request(request: &PublicationPlanRequest) -> Vec<PublicationPlanBlocker> {
    let mut blockers = Vec::new();
    validate_destination(request.destination, &mut blockers);
    validate_release_id(&request.release_id, &mut blockers);
    validate_count(
        request.artifacts.len(),
        RELEASE_PUBLICATION_ARTIFACTS_COUNT_MAX,
        PublicationPlanBlockerKind::ArtifactCountOverflow,
        PublicationPlanBlockerKind::ArtifactCountExceeded,
        "artifact",
        &mut blockers,
    );
    validate_count(
        request.policy.len(),
        RELEASE_PUBLICATION_POLICY_FACTS_COUNT_MAX,
        PublicationPlanBlockerKind::PolicyCountOverflow,
        PublicationPlanBlockerKind::PolicyCountExceeded,
        "policy fact",
        &mut blockers,
    );
    if request.artifacts.is_empty() {
        blockers.push(blocker(PublicationPlanBlockerKind::EmptyArtifacts, None, "publication plan requires artifacts"));
    }
    if request.policy.is_empty() {
        blockers.push(blocker(PublicationPlanBlockerKind::EmptyPolicy, None, "publication plan requires policy facts"));
    }
    for artifact in &request.artifacts {
        validate_artifact(artifact, &mut blockers);
    }
    for fact in &request.policy {
        validate_policy_fact(fact, &mut blockers);
    }
    blockers
}

// r[impl mantle.release_provenance.bundle_publication.staging_validation]
pub fn validate_staged_publication_artifacts(
    plan: &PublicationPlan,
    artifacts: Vec<PublicationArtifactInput>,
) -> Result<(), Vec<PublicationPlanBlocker>> {
    let staged = plan_release_publication(PublicationPlanRequest {
        release_id: plan.release_id.clone(),
        destination: PublicationDestinationObservation::Absent,
        artifacts,
        policy: plan.policy.clone(),
    })?;
    if staged.plan_identity_blake3 != plan.plan_identity_blake3 {
        return Err(vec![blocker(
            PublicationPlanBlockerKind::StagedArtifactMismatch,
            None,
            "staged release artifacts do not match the pre-mutation publication plan",
        )]);
    }
    assert_eq!(staged.artifacts, plan.artifacts);
    assert_eq!(staged.policy, plan.policy);
    Ok(())
}

fn validate_destination(destination: PublicationDestinationObservation, blockers: &mut Vec<PublicationPlanBlocker>) {
    if destination == PublicationDestinationObservation::Absent {
        return;
    }
    blockers.push(blocker(
        PublicationPlanBlockerKind::DestinationExists,
        Some(format!("{destination:?}")),
        &format!("release publication destination must be absent, found {destination:?}"),
    ));
}

fn validate_release_id(release_id: &str, blockers: &mut Vec<PublicationPlanBlocker>) {
    if release_id.trim().is_empty() {
        blockers.push(blocker(
            PublicationPlanBlockerKind::EmptyReleaseId,
            None,
            "publication release id must not be empty",
        ));
        return;
    }
    if text_len_u32(release_id).is_none_or(|length| length > RELEASE_PUBLICATION_TEXT_BYTES_MAX) {
        blockers.push(blocker(
            PublicationPlanBlockerKind::ReleaseIdTooLong,
            Some(release_id.to_string()),
            "publication release id exceeds the text bound",
        ));
    }
}

fn validate_count(
    count: usize,
    count_max: u32,
    overflow_kind: PublicationPlanBlockerKind,
    exceeded_kind: PublicationPlanBlockerKind,
    label: &str,
    blockers: &mut Vec<PublicationPlanBlocker>,
) {
    let Ok(count_u32) = u32::try_from(count) else {
        blockers.push(blocker(overflow_kind, None, &format!("publication {label} count overflowed u32")));
        return;
    };
    if count_u32 > count_max {
        blockers.push(blocker(
            exceeded_kind,
            None,
            &format!("publication {label} count {count_u32} exceeds {count_max}"),
        ));
    }
}

fn validate_artifact(artifact: &PublicationArtifactInput, blockers: &mut Vec<PublicationPlanBlocker>) {
    validate_artifact_path(&artifact.relative_path, blockers);
    if !is_lower_blake3(&artifact.digest_blake3) {
        blockers.push(blocker(
            PublicationPlanBlockerKind::InvalidArtifactDigest,
            Some(artifact.relative_path.clone()),
            &format!("publication artifact {} has an invalid BLAKE3 digest", artifact.relative_path),
        ));
    }
}

fn validate_artifact_path(path: &str, blockers: &mut Vec<PublicationPlanBlocker>) {
    if path == MANIFEST_RELATIVE_PATH {
        blockers.push(blocker(
            PublicationPlanBlockerKind::ManifestPathReserved,
            Some(path.to_string()),
            "publication artifact layout reserves manifest.json for the manifest-last phase",
        ));
        return;
    }
    if !is_normal_relative_path(path) {
        blockers.push(blocker(
            PublicationPlanBlockerKind::InvalidArtifactPath,
            Some(path.to_string()),
            &format!("publication artifact path must be normalized and relative: {path}"),
        ));
        return;
    }
    if text_len_u32(path).is_none_or(|length| length > RELEASE_PUBLICATION_TEXT_BYTES_MAX) {
        blockers.push(blocker(
            PublicationPlanBlockerKind::ArtifactPathTooLong,
            Some(path.to_string()),
            &format!("publication artifact path exceeds {} bytes: {path}", RELEASE_PUBLICATION_TEXT_BYTES_MAX),
        ));
    }
    let component_count = path.split(PATH_SEPARATOR).count();
    let component_count_exceeded = match u32::try_from(component_count) {
        Ok(count) => count > RELEASE_PUBLICATION_PATH_COMPONENTS_MAX,
        Err(_) => true,
    };
    if component_count_exceeded {
        blockers.push(blocker(
            PublicationPlanBlockerKind::ArtifactPathTooDeep,
            Some(path.to_string()),
            &format!(
                "publication artifact path exceeds {} components: {path}",
                RELEASE_PUBLICATION_PATH_COMPONENTS_MAX
            ),
        ));
    }
}

fn validate_policy_fact(fact: &PublicationPolicyFact, blockers: &mut Vec<PublicationPlanBlocker>) {
    if fact.name.trim().is_empty() {
        blockers.push(blocker(
            PublicationPlanBlockerKind::EmptyPolicyName,
            None,
            "publication policy fact name must not be empty",
        ));
    }
    if text_len_u32(&fact.name).is_none_or(|length| length > RELEASE_PUBLICATION_TEXT_BYTES_MAX) {
        blockers.push(blocker(
            PublicationPlanBlockerKind::PolicyNameTooLong,
            Some(fact.name.clone()),
            "publication policy fact name exceeds the text bound",
        ));
    }
    if text_len_u32(&fact.value).is_none_or(|length| length > RELEASE_PUBLICATION_TEXT_BYTES_MAX) {
        blockers.push(blocker(
            PublicationPlanBlockerKind::PolicyValueTooLong,
            Some(fact.name.clone()),
            "publication policy fact value exceeds the text bound",
        ));
    }
}

fn validate_sorted_artifact_relationships(artifacts: &[PublicationArtifactInput]) -> Vec<PublicationPlanBlocker> {
    let mut blockers = Vec::new();
    for pair in artifacts.windows(WINDOW_PAIR_COUNT) {
        let left = &pair[0].relative_path;
        let right = &pair[1].relative_path;
        if left == right {
            blockers.push(blocker(
                PublicationPlanBlockerKind::DuplicateArtifactPath,
                Some(left.clone()),
                &format!("publication artifact layout contains duplicate path {left}"),
            ));
            continue;
        }
        let descendant_prefix = format!("{left}{PATH_SEPARATOR}");
        if right.starts_with(&descendant_prefix) {
            blockers.push(blocker(
                PublicationPlanBlockerKind::OverlappingArtifactPath,
                Some(right.clone()),
                &format!("publication artifact path {right} overlaps planned artifact root {left}"),
            ));
        }
    }
    blockers
}

fn validate_sorted_policy_relationships(policy: &[PublicationPolicyFact]) -> Vec<PublicationPlanBlocker> {
    let mut blockers = Vec::new();
    for pair in policy.windows(WINDOW_PAIR_COUNT) {
        if pair[0].name == pair[1].name {
            blockers.push(blocker(
                PublicationPlanBlockerKind::DuplicatePolicyName,
                Some(pair[0].name.clone()),
                &format!("publication policy contains duplicate fact name {}", pair[0].name),
            ));
        }
    }
    blockers
}

#[derive(Serialize)]
struct PublicationPlanIdentity<'a> {
    schema: &'static str,
    release_id: &'a str,
    artifacts: &'a [PublicationArtifactInput],
    policy: &'a [PublicationPolicyFact],
}

fn publication_plan_identity(
    release_id: &str,
    artifacts: &[PublicationArtifactInput],
    policy: &[PublicationPolicyFact],
) -> String {
    let identity = PublicationPlanIdentity {
        schema: RELEASE_PUBLICATION_PLAN_SCHEMA,
        release_id,
        artifacts,
        policy,
    };
    let bytes = serde_json::to_vec(&identity).expect("publication plan identity serialization must succeed");
    assert!(!bytes.is_empty(), "publication plan identity bytes must not be empty");
    assert!(!artifacts.is_empty(), "publication plan identity requires artifacts");
    blake3::hash(&bytes).to_hex().to_string()
}

fn is_normal_relative_path(path: &str) -> bool {
    if path.is_empty() || path.starts_with(PATH_SEPARATOR) {
        return false;
    }
    if path.contains(WINDOWS_PATH_SEPARATOR) || path.contains(NUL_CHARACTER) {
        return false;
    }
    path.split(PATH_SEPARATOR)
        .all(|component| !component.is_empty() && component != "." && component != "..")
}

fn is_lower_blake3(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH_CHARS
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn text_len_u32(value: &str) -> Option<u32> {
    u32::try_from(value.len()).ok()
}

fn blocker(kind: PublicationPlanBlockerKind, subject: Option<String>, message: &str) -> PublicationPlanBlocker {
    PublicationPlanBlocker {
        kind,
        subject,
        message: message.to_string(),
    }
}

fn sort_blockers(blockers: &mut [PublicationPlanBlocker]) {
    blockers.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.subject.cmp(&right.subject))
            .then_with(|| left.message.cmp(&right.message))
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationPhase {
    Planned,
    StageCreated,
    InputsCopied,
    ArtifactsHashed,
    ManifestSerialized,
    ManifestWritten,
    Verified,
    Failed,
    Committed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationFailurePhase {
    StageCreation,
    InputCopy,
    ArtifactHashing,
    ManifestSerialization,
    ManifestWrite,
    Verification,
    Cleanup,
    PreCommit,
    CommitRace,
    Commit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationEvent {
    StageCreated,
    InputsCopied,
    ArtifactsHashed,
    ManifestSerialized,
    ManifestWritten,
    VerificationAccepted,
    VerificationRejected,
    CommitSucceeded,
    CommitLostRace,
    Failed(PublicationFailurePhase),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationState {
    pub plan_identity_blake3: String,
    pub phase: PublicationPhase,
    pub failures: Vec<PublicationFailurePhase>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicationTransitionError {
    pub phase: PublicationPhase,
    pub event: PublicationEvent,
}

pub fn initial_publication_state(plan: &PublicationPlan) -> PublicationState {
    assert_eq!(plan.schema, RELEASE_PUBLICATION_PLAN_SCHEMA);
    assert!(is_lower_blake3(&plan.plan_identity_blake3));
    PublicationState {
        plan_identity_blake3: plan.plan_identity_blake3.clone(),
        phase: PublicationPhase::Planned,
        failures: Vec::new(),
    }
}

// r[impl mantle.release_provenance.bundle_publication.boundary.test]
pub fn transition_publication_state(
    mut state: PublicationState,
    event: PublicationEvent,
) -> Result<PublicationState, PublicationTransitionError> {
    if let PublicationEvent::Failed(failure) = event {
        return transition_failure(state, failure, event);
    }
    let next = match (state.phase, event) {
        (PublicationPhase::Planned, PublicationEvent::StageCreated) => PublicationPhase::StageCreated,
        (PublicationPhase::StageCreated, PublicationEvent::InputsCopied) => PublicationPhase::InputsCopied,
        (PublicationPhase::InputsCopied, PublicationEvent::ArtifactsHashed) => PublicationPhase::ArtifactsHashed,
        (PublicationPhase::ArtifactsHashed, PublicationEvent::ManifestSerialized) => {
            PublicationPhase::ManifestSerialized
        }
        (PublicationPhase::ManifestSerialized, PublicationEvent::ManifestWritten) => PublicationPhase::ManifestWritten,
        (PublicationPhase::ManifestWritten, PublicationEvent::VerificationAccepted) => PublicationPhase::Verified,
        (PublicationPhase::ManifestWritten, PublicationEvent::VerificationRejected) => {
            return transition_failure(state, PublicationFailurePhase::Verification, event);
        }
        (PublicationPhase::Verified, PublicationEvent::CommitSucceeded) => PublicationPhase::Committed,
        (PublicationPhase::Verified, PublicationEvent::CommitLostRace) => {
            return transition_failure(state, PublicationFailurePhase::CommitRace, event);
        }
        _ => {
            return Err(PublicationTransitionError {
                phase: state.phase,
                event,
            });
        }
    };
    state.phase = next;
    assert!(state.failures.is_empty(), "successful publication transitions cannot carry failures");
    assert_ne!(state.phase, PublicationPhase::Failed);
    Ok(state)
}

fn transition_failure(
    mut state: PublicationState,
    failure: PublicationFailurePhase,
    event: PublicationEvent,
) -> Result<PublicationState, PublicationTransitionError> {
    if state.phase == PublicationPhase::Committed {
        return Err(PublicationTransitionError {
            phase: state.phase,
            event,
        });
    }
    if state.phase == PublicationPhase::Failed && failure != PublicationFailurePhase::Cleanup {
        return Err(PublicationTransitionError {
            phase: state.phase,
            event,
        });
    }
    let failure_count = u32::try_from(state.failures.len()).map_err(|_| PublicationTransitionError {
        phase: state.phase,
        event,
    })?;
    if failure_count >= RELEASE_PUBLICATION_FAILURES_COUNT_MAX {
        return Err(PublicationTransitionError {
            phase: state.phase,
            event,
        });
    }
    state.failures.push(failure);
    state.phase = PublicationPhase::Failed;
    assert!(!state.failures.is_empty(), "failed publication state must carry a failure");
    assert!(!publication_commit_eligible(&state));
    Ok(state)
}

pub fn publication_commit_eligible(state: &PublicationState) -> bool {
    state.phase == PublicationPhase::Verified && state.failures.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    const RELEASE_ID: &str = "mantle-test-release";
    const SOURCE_PATH: &str = "source/mantle-src.tar";
    const BINARY_PATH: &str = "binaries/01-mantle";
    const PROOF_PATH: &str = "proof/self-hosting";
    const INVENTORY_PATH: &str = "proof/inventory.md";
    const POLICY_NAME: &str = "workflow-version";
    const POLICY_VALUE: &str = "mantle-release-proof-v1";
    const FIRST_DIGEST_BYTE: u8 = 1;
    const SECOND_DIGEST_BYTE: u8 = 2;
    const ARTIFACT_SIZE_BYTES: u64 = 7;
    const EXISTING_DESTINATION_CASES_COUNT: usize = 5;

    fn digest(byte: u8) -> String {
        format!("{byte:02x}").repeat(BLAKE3_HEX_LENGTH_CHARS / WINDOW_PAIR_COUNT)
    }

    fn artifact(path: &str, kind: BundledArtifactKind, byte: u8) -> PublicationArtifactInput {
        PublicationArtifactInput {
            relative_path: path.to_string(),
            kind,
            size_bytes: ARTIFACT_SIZE_BYTES,
            digest_blake3: digest(byte),
        }
    }

    fn plan_request() -> PublicationPlanRequest {
        PublicationPlanRequest {
            release_id: RELEASE_ID.to_string(),
            destination: PublicationDestinationObservation::Absent,
            artifacts: vec![
                artifact(PROOF_PATH, BundledArtifactKind::Directory, FIRST_DIGEST_BYTE),
                artifact(BINARY_PATH, BundledArtifactKind::File, SECOND_DIGEST_BYTE),
                artifact(SOURCE_PATH, BundledArtifactKind::File, FIRST_DIGEST_BYTE),
                artifact(INVENTORY_PATH, BundledArtifactKind::File, SECOND_DIGEST_BYTE),
            ],
            policy: vec![PublicationPolicyFact {
                name: POLICY_NAME.to_string(),
                value: POLICY_VALUE.to_string(),
            }],
        }
    }

    // r[verify mantle.release_provenance.bundle_publication.boundary.test]
    #[test]
    fn plan_identity_is_deterministic_and_has_no_stage_name_input() {
        let mut reordered = plan_request();
        reordered.artifacts.reverse();
        let first = plan_release_publication(plan_request()).expect("first plan");
        let second = plan_release_publication(reordered).expect("reordered plan");

        assert_eq!(first.plan_identity_blake3, second.plan_identity_blake3);
        assert_eq!(first.artifacts, second.artifacts);
        assert_eq!(first.plan_identity_blake3.len(), BLAKE3_HEX_LENGTH_CHARS);
        assert!(!first.plan_identity_blake3.contains("stage"));
    }

    #[test]
    fn plan_identity_changes_with_release_layout_input_or_policy() {
        let baseline = plan_release_publication(plan_request()).expect("baseline plan");
        let mut release = plan_request();
        release.release_id.push_str("-other");
        let mut layout = plan_request();
        layout.artifacts[0].relative_path = "proof/other".to_string();
        let mut input = plan_request();
        input.artifacts[0].digest_blake3 = digest(SECOND_DIGEST_BYTE);
        let mut policy = plan_request();
        policy.policy[0].value.push_str("-other");

        let identities = [release, layout, input, policy]
            .into_iter()
            .map(|request| plan_release_publication(request).expect("variant plan").plan_identity_blake3)
            .collect::<Vec<_>>();

        assert!(identities.iter().all(|identity| identity != &baseline.plan_identity_blake3));
        assert_eq!(identities.len(), identities.iter().collect::<alloc::collections::BTreeSet<_>>().len());
    }

    #[test]
    fn preexisting_destination_shapes_are_all_blockers() {
        let cases: [PublicationDestinationObservation; EXISTING_DESTINATION_CASES_COUNT] = [
            PublicationDestinationObservation::File,
            PublicationDestinationObservation::Symlink,
            PublicationDestinationObservation::EmptyDirectory,
            PublicationDestinationObservation::NonEmptyDirectory,
            PublicationDestinationObservation::Other,
        ];

        for destination in cases {
            let mut request = plan_request();
            request.destination = destination;
            let blockers = plan_release_publication(request).expect_err("existing destination must block");
            assert_eq!(blockers.len(), 1, "{destination:?}");
            assert_eq!(blockers[0].kind, PublicationPlanBlockerKind::DestinationExists, "{destination:?}");
        }
    }

    #[test]
    fn invalid_layout_and_policy_fail_closed() {
        let mut request = plan_request();
        request
            .artifacts
            .push(artifact(MANIFEST_RELATIVE_PATH, BundledArtifactKind::File, FIRST_DIGEST_BYTE));
        request.artifacts.push(artifact(SOURCE_PATH, BundledArtifactKind::File, SECOND_DIGEST_BYTE));
        request.artifacts.push(artifact("../escape", BundledArtifactKind::File, FIRST_DIGEST_BYTE));
        request.policy.push(PublicationPolicyFact {
            name: POLICY_NAME.to_string(),
            value: "duplicate".to_string(),
        });

        let blockers = plan_release_publication(request).expect_err("invalid plan must fail");
        let kinds = blockers.iter().map(|blocker| blocker.kind).collect::<Vec<_>>();

        assert!(kinds.contains(&PublicationPlanBlockerKind::ManifestPathReserved));
        assert!(kinds.contains(&PublicationPlanBlockerKind::DuplicateArtifactPath));
        assert!(kinds.contains(&PublicationPlanBlockerKind::InvalidArtifactPath));
        assert!(kinds.contains(&PublicationPlanBlockerKind::DuplicatePolicyName));
    }

    // r[verify mantle.release_provenance.bundle_publication.boundary.test]
    #[test]
    fn staged_artifacts_must_match_the_pre_mutation_plan() {
        let plan = plan_release_publication(plan_request()).expect("valid plan");
        let matching = validate_staged_publication_artifacts(&plan, plan.artifacts.clone());
        let mut changed = plan.artifacts.clone();
        changed[0].digest_blake3 = digest(FIRST_DIGEST_BYTE);
        if changed[0].digest_blake3 == plan.artifacts[0].digest_blake3 {
            changed[0].digest_blake3 = digest(SECOND_DIGEST_BYTE);
        }
        let mismatch = validate_staged_publication_artifacts(&plan, changed).expect_err("staged drift must fail");

        assert!(matching.is_ok());
        assert_eq!(mismatch.len(), 1);
        assert_eq!(mismatch[0].kind, PublicationPlanBlockerKind::StagedArtifactMismatch);
    }

    #[test]
    fn state_machine_accepts_only_the_verified_commit_sequence() {
        let plan = plan_release_publication(plan_request()).expect("valid plan");
        let events = [
            PublicationEvent::StageCreated,
            PublicationEvent::InputsCopied,
            PublicationEvent::ArtifactsHashed,
            PublicationEvent::ManifestSerialized,
            PublicationEvent::ManifestWritten,
            PublicationEvent::VerificationAccepted,
        ];
        let mut state = initial_publication_state(&plan);
        for event in events {
            state = transition_publication_state(state, event).expect("valid transition");
        }

        assert_eq!(state.phase, PublicationPhase::Verified);
        assert!(publication_commit_eligible(&state));
        state = transition_publication_state(state, PublicationEvent::CommitSucceeded).expect("commit transition");
        assert_eq!(state.phase, PublicationPhase::Committed);
        assert!(!publication_commit_eligible(&state));
    }

    #[test]
    fn state_machine_rejects_skips_and_post_commit_failures() {
        let plan = plan_release_publication(plan_request()).expect("valid plan");
        let initial = initial_publication_state(&plan);
        let skipped = transition_publication_state(initial.clone(), PublicationEvent::InputsCopied)
            .expect_err("phase skip must fail");
        let committed = [
            PublicationEvent::StageCreated,
            PublicationEvent::InputsCopied,
            PublicationEvent::ArtifactsHashed,
            PublicationEvent::ManifestSerialized,
            PublicationEvent::ManifestWritten,
            PublicationEvent::VerificationAccepted,
            PublicationEvent::CommitSucceeded,
        ]
        .into_iter()
        .try_fold(initial, transition_publication_state)
        .expect("committed state");
        let post_commit =
            transition_publication_state(committed, PublicationEvent::Failed(PublicationFailurePhase::Cleanup))
                .expect_err("committed publication is terminal");

        assert_eq!(skipped.phase, PublicationPhase::Planned);
        assert_eq!(post_commit.phase, PublicationPhase::Committed);
    }

    #[test]
    fn verification_rejection_and_commit_race_are_terminal_failures() {
        let plan = plan_release_publication(plan_request()).expect("valid plan");
        let manifest_written = [
            PublicationEvent::StageCreated,
            PublicationEvent::InputsCopied,
            PublicationEvent::ArtifactsHashed,
            PublicationEvent::ManifestSerialized,
            PublicationEvent::ManifestWritten,
        ]
        .into_iter()
        .try_fold(initial_publication_state(&plan), transition_publication_state)
        .expect("manifest-written state");
        let verification_failed =
            transition_publication_state(manifest_written.clone(), PublicationEvent::VerificationRejected)
                .expect("verification failure state");
        let verified = transition_publication_state(manifest_written, PublicationEvent::VerificationAccepted)
            .expect("verified state");
        let race_failed = transition_publication_state(verified, PublicationEvent::CommitLostRace)
            .expect("commit race failure state");

        assert_eq!(verification_failed.failures, vec![PublicationFailurePhase::Verification]);
        assert_eq!(race_failed.failures, vec![PublicationFailurePhase::CommitRace]);
        assert!(!publication_commit_eligible(&verification_failed));
        assert!(!publication_commit_eligible(&race_failed));
    }

    #[test]
    fn cleanup_failure_can_augment_one_phase_failure_only() {
        let plan = plan_release_publication(plan_request()).expect("valid plan");
        let failed = transition_publication_state(
            initial_publication_state(&plan),
            PublicationEvent::Failed(PublicationFailurePhase::StageCreation),
        )
        .expect("stage failure");
        let cleanup_failed =
            transition_publication_state(failed, PublicationEvent::Failed(PublicationFailurePhase::Cleanup))
                .expect("cleanup failure");
        let overflow = transition_publication_state(
            cleanup_failed.clone(),
            PublicationEvent::Failed(PublicationFailurePhase::Cleanup),
        )
        .expect_err("failure bound must hold");

        assert_eq!(cleanup_failed.failures.len(), RELEASE_PUBLICATION_FAILURES_COUNT_MAX as usize);
        assert_eq!(overflow.phase, PublicationPhase::Failed);
    }
}
