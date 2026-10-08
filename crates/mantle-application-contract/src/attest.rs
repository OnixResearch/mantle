//! Bounded, request-bound attestation effects and observations.
//!
//! A store access is composite: opening the store can initialize identity and
//! looking up a closure can persist an attestation. The store API reports no
//! optional-write receipt. A returned call is not proof of those writes.

/// Every executable attestation operation has a distinct request identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttestOperation {
    Show,
    Closure,
    VerifyArtifact,
    VerifyClosure,
    VerifyProject,
    Diff,
    Project,
    ReleaseShow,
    KeyShow,
    WitnessCreate,
    WitnessShow,
    WitnessImport,
    PolicyInit,
    ReleaseVerify,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttestCapability {
    /// Includes optional store identity initialization and closure persistence.
    StoreAccess,
    ReadFiles,
    /// May read inputs, publish verification files, and for witness creation
    /// may initialize a configured signing key. No durability is inferred.
    PublishFiles,
}

/// Process-local identity of a selected path or ordered typed input tuple.
/// The adapter hashes the actual port arguments; this is not a persisted
/// content digest or proof that a file or optional store write exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttestIdentity(pub [u8; 32]);

/// Authority selected before executing the first port. The working and config
/// directories bind relative project, trust, and signing-key inputs; state,
/// output, backend and bases bind every store-backed action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttestAuthority {
    pub current_dir: AttestIdentity,
    pub config_dir: Option<AttestIdentity>,
    pub state: AttestIdentity,
    pub output: AttestIdentity,
    pub store_backend: u8,
    pub base_states: AttestIdentity,
}

/// The shape and complete identity of one selected operation's inputs. A
/// digest binds all ordered selectors, source files and policy options without
/// copying strings or leaking key material into a report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttestTarget {
    StoreSelector(AttestIdentity),
    StoreRoots(AttestIdentity),
    StorePair(AttestIdentity),
    ProjectVerification(AttestIdentity),
    VerificationDirectory(AttestIdentity),
    SigningKey(AttestIdentity),
    WitnessCreate(AttestIdentity),
    WitnessShow(AttestIdentity),
    WitnessImport(AttestIdentity),
    PolicyInit(AttestIdentity),
    ReleaseVerification(AttestIdentity),
}

impl AttestTarget {
    pub const fn identity(self) -> AttestIdentity {
        match self {
            Self::StoreSelector(id)
            | Self::StoreRoots(id)
            | Self::StorePair(id)
            | Self::ProjectVerification(id)
            | Self::VerificationDirectory(id)
            | Self::SigningKey(id)
            | Self::WitnessCreate(id)
            | Self::WitnessShow(id)
            | Self::WitnessImport(id)
            | Self::PolicyInit(id)
            | Self::ReleaseVerification(id) => id,
        }
    }
}

/// Exact selected input usage and a ceiling on returned result entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttestLimits {
    pub selected_inputs: u32,
    pub selected_bytes: u32,
    pub min_result_entries: u32,
    pub max_result_entries: u32,
}

pub const MAX_ATTEST_SELECTED_INPUTS: u32 = 1_024;
pub const MAX_ATTEST_SELECTED_BYTES: u32 = 65_536;
pub const MAX_ATTEST_RESULT_ENTRIES: u32 = 2_048;
pub const MAX_ATTEST_EFFECTS: usize = 1;

/// The selected output authority is an operation-specific selected path
/// (store state, read input, or publication directory), not a declaration
/// that an optional write occurred.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttestRequest {
    pub operation: AttestOperation,
    pub authority: AttestAuthority,
    pub target: AttestTarget,
    pub limits: AttestLimits,
    pub output_authority: AttestIdentity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttestEffect {
    pub index: u8,
    pub effect_id: AttestIdentity,
    pub capability: AttestCapability,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttestPlan {
    pub request: AttestRequest,
    pub effects: [AttestEffect; MAX_ATTEST_EFFECTS],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttestProgress {
    StoreOpenFailed,
    StoreOpenedThenFailed,
    StoreAccessReturned,
    ReadFailed,
    InputRejected,
    ReadRejected,
    ReadReturned,
    PublishFailed,
    PublishReturned,
}

/// Codes are tied to the real port stage, never to an error's presentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttestFailure {
    StoreOpen,
    /// Store already opened; selection, lookup, verification or later reads failed.
    StoreContinuation,
    /// Read-only adapter call failed while loading or verifying selected files.
    ReadCall,
    Input,
    ReadValidation,
    /// Composite publication call failed during input reads or writes; partial output is possible.
    PublicationCall,
}

/// Actual selected invocation and returned port receipt. Returned entry count
/// is an adapter result, not a count of optional writes or durable bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttestObservation {
    pub operation: AttestOperation,
    pub effect_index: u8,
    pub effect_id: AttestIdentity,
    pub capability: AttestCapability,
    pub authority: AttestAuthority,
    pub target: AttestTarget,
    pub selected_inputs: u32,
    pub selected_bytes: u32,
    pub result_entries: u32,
    pub output_authority: AttestIdentity,
    pub progress: AttestProgress,
    pub failure: Option<AttestFailure>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttestOutcome {
    Completed,
    Failed,
    Rejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttestPlanBlocker {
    InputLimit,
    ResultLimit,
    WrongTarget,
}

/// Pure admission before any filesystem, store or network port. Limits are
/// deliberately finite and classify actual usage independently of the plan.
pub fn plan_attest(request: AttestRequest) -> Result<AttestPlan, AttestPlanBlocker> {
    if request.limits.selected_inputs > MAX_ATTEST_SELECTED_INPUTS
        || request.limits.selected_bytes > MAX_ATTEST_SELECTED_BYTES
    {
        return Err(AttestPlanBlocker::InputLimit);
    }
    if request.limits.max_result_entries > MAX_ATTEST_RESULT_ENTRIES
        || request.limits.min_result_entries > request.limits.max_result_entries
    {
        return Err(AttestPlanBlocker::ResultLimit);
    }
    let (expected_target, capability, minimum_results, maximum_results) = match request.operation {
        AttestOperation::Show | AttestOperation::VerifyArtifact => (0, AttestCapability::StoreAccess, 1, 1),
        AttestOperation::Closure | AttestOperation::VerifyClosure | AttestOperation::Project => {
            (1, AttestCapability::StoreAccess, 1, 1)
        }
        AttestOperation::Diff => (2, AttestCapability::StoreAccess, 2, 2),
        AttestOperation::VerifyProject => (3, AttestCapability::StoreAccess, 1, 1),
        AttestOperation::ReleaseShow => (4, AttestCapability::ReadFiles, 1, 1),
        AttestOperation::KeyShow => (5, AttestCapability::ReadFiles, 1, 1),
        AttestOperation::WitnessCreate => (6, AttestCapability::PublishFiles, 2, 2),
        AttestOperation::WitnessShow => (7, AttestCapability::ReadFiles, 0, 1_024),
        AttestOperation::WitnessImport => (8, AttestCapability::PublishFiles, 0, 2_048),
        AttestOperation::PolicyInit => (9, AttestCapability::PublishFiles, 2, 2),
        AttestOperation::ReleaseVerify => (10, AttestCapability::ReadFiles, 1, 1),
    };
    if request.limits.min_result_entries != minimum_results || request.limits.max_result_entries != maximum_results {
        return Err(AttestPlanBlocker::ResultLimit);
    }
    let target_kind = match request.target {
        AttestTarget::StoreSelector(_) => 0,
        AttestTarget::StoreRoots(_) => 1,
        AttestTarget::StorePair(_) => 2,
        AttestTarget::ProjectVerification(_) => 3,
        AttestTarget::VerificationDirectory(_) => 4,
        AttestTarget::SigningKey(_) => 5,
        AttestTarget::WitnessCreate(_) => 6,
        AttestTarget::WitnessShow(_) => 7,
        AttestTarget::WitnessImport(_) => 8,
        AttestTarget::PolicyInit(_) => 9,
        AttestTarget::ReleaseVerification(_) => 10,
    };
    if target_kind != expected_target {
        return Err(AttestPlanBlocker::WrongTarget);
    }
    Ok(AttestPlan {
        request,
        effects: [AttestEffect {
            index: 0,
            effect_id: request.target.identity(),
            capability,
        }],
    })
}

/// Wrong effect, authority, exact target, output directory or limit usage is
/// an execution blocker; no success evidence can be fabricated from it.
pub fn classify_attest(plan: AttestPlan, observation: AttestObservation) -> AttestOutcome {
    let Ok(expected) = plan_attest(plan.request) else {
        return AttestOutcome::Rejected;
    };
    if plan != expected
        || observation.effect_index != expected.effects[0].index
        || observation.effect_id != expected.effects[0].effect_id
        || observation.capability != expected.effects[0].capability
        || observation.operation != plan.request.operation
        || observation.authority != plan.request.authority
        || observation.target != plan.request.target
        || observation.selected_inputs != plan.request.limits.selected_inputs
        || observation.selected_bytes != plan.request.limits.selected_bytes
        || observation.result_entries > plan.request.limits.max_result_entries
        || (observation.failure.is_none() && observation.result_entries < plan.request.limits.min_result_entries)
        || observation.output_authority != plan.request.output_authority
    {
        return AttestOutcome::Rejected;
    }
    match (observation.capability, observation.progress, observation.failure) {
        (AttestCapability::StoreAccess, AttestProgress::StoreAccessReturned, None)
        | (AttestCapability::ReadFiles, AttestProgress::ReadReturned, None)
        | (AttestCapability::PublishFiles, AttestProgress::PublishReturned, None) => AttestOutcome::Completed,
        (AttestCapability::StoreAccess, AttestProgress::StoreOpenFailed, Some(AttestFailure::StoreOpen))
        | (
            AttestCapability::StoreAccess,
            AttestProgress::StoreOpenedThenFailed,
            Some(AttestFailure::StoreContinuation),
        )
        | (AttestCapability::ReadFiles, AttestProgress::ReadFailed, Some(AttestFailure::ReadCall))
        | (AttestCapability::ReadFiles, AttestProgress::InputRejected, Some(AttestFailure::Input))
        | (AttestCapability::ReadFiles, AttestProgress::ReadRejected, Some(AttestFailure::ReadValidation))
        | (AttestCapability::PublishFiles, AttestProgress::PublishFailed, Some(AttestFailure::PublicationCall)) => {
            AttestOutcome::Failed
        }
        _ => AttestOutcome::Rejected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (AttestPlan, AttestObservation) {
        let path = AttestIdentity([1; 32]);
        let authority = AttestAuthority {
            current_dir: AttestIdentity([9; 32]),
            config_dir: None,
            state: path,
            output: AttestIdentity([2; 32]),
            store_backend: 1,
            base_states: AttestIdentity([0; 32]),
        };
        let request = AttestRequest {
            operation: AttestOperation::Show,
            authority,
            target: AttestTarget::StoreSelector(AttestIdentity([3; 32])),
            limits: AttestLimits {
                selected_inputs: 1,
                selected_bytes: 12,
                min_result_entries: 1,
                max_result_entries: 1,
            },
            output_authority: path,
        };
        let plan = plan_attest(request).unwrap();
        let observation = AttestObservation {
            operation: AttestOperation::Show,
            effect_index: 0,
            effect_id: request.target.identity(),
            capability: AttestCapability::StoreAccess,
            authority,
            target: request.target,
            selected_inputs: 1,
            selected_bytes: 12,
            result_entries: 1,
            output_authority: path,
            progress: AttestProgress::StoreAccessReturned,
            failure: None,
        };
        (plan, observation)
    }

    #[test]
    fn wrong_target_authority_output_or_limit_never_completes() {
        let (plan, observation) = fixture();
        assert_eq!(classify_attest(plan, observation), AttestOutcome::Completed);
        // The observation is produced at the port boundary, not synthesized
        // from the plan; every independent fact must be checked.
        for invalid in [
            AttestObservation {
                effect_id: AttestIdentity([8; 32]),
                ..observation
            },
            AttestObservation {
                effect_index: 1,
                ..observation
            },
            AttestObservation {
                capability: AttestCapability::ReadFiles,
                ..observation
            },
            AttestObservation {
                operation: AttestOperation::VerifyArtifact,
                ..observation
            },
            AttestObservation {
                target: AttestTarget::StoreSelector(AttestIdentity([4; 32])),
                ..observation
            },
            AttestObservation {
                authority: AttestAuthority {
                    current_dir: AttestIdentity([10; 32]),
                    ..observation.authority
                },
                ..observation
            },
            AttestObservation {
                authority: AttestAuthority {
                    config_dir: Some(AttestIdentity([11; 32])),
                    ..observation.authority
                },
                ..observation
            },
            AttestObservation {
                authority: AttestAuthority {
                    state: AttestIdentity([5; 32]),
                    ..observation.authority
                },
                ..observation
            },
            AttestObservation {
                output_authority: AttestIdentity([6; 32]),
                ..observation
            },
            AttestObservation {
                result_entries: 0,
                ..observation
            },
            AttestObservation {
                result_entries: 2,
                ..observation
            },
            AttestObservation {
                selected_inputs: 2,
                ..observation
            },
            AttestObservation {
                selected_bytes: 13,
                ..observation
            },
        ] {
            assert_eq!(classify_attest(plan, invalid), AttestOutcome::Rejected);
        }
    }

    #[test]
    fn actual_failure_stage_and_code_are_required() {
        let (plan, observation) = fixture();
        let failed = AttestObservation {
            result_entries: 0,
            progress: AttestProgress::StoreOpenFailed,
            failure: Some(AttestFailure::StoreOpen),
            ..observation
        };
        assert_eq!(classify_attest(plan, failed), AttestOutcome::Failed);
        assert_eq!(
            classify_attest(plan, AttestObservation {
                failure: Some(AttestFailure::PublicationCall),
                ..failed
            }),
            AttestOutcome::Rejected
        );
    }

    #[test]
    fn bounded_admission_rejects_wrong_shape_and_oversized_selection() {
        let (plan, _) = fixture();
        let request = AttestRequest {
            limits: AttestLimits {
                selected_inputs: MAX_ATTEST_SELECTED_INPUTS + 1,
                ..plan.request.limits
            },
            ..plan.request
        };
        assert_eq!(plan_attest(request), Err(AttestPlanBlocker::InputLimit));
        assert_eq!(
            plan_attest(AttestRequest {
                limits: AttestLimits {
                    min_result_entries: 0,
                    ..plan.request.limits
                },
                ..plan.request
            }),
            Err(AttestPlanBlocker::ResultLimit),
        );
        assert_eq!(
            plan_attest(AttestRequest {
                target: AttestTarget::StoreRoots(AttestIdentity([7; 32])),
                ..plan.request
            }),
            Err(AttestPlanBlocker::WrongTarget),
        );
    }
}
