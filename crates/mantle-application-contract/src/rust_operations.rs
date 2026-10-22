//! Rust toolchain, bundle, test and explicit-profile operation boundaries.
//!
//! Plans admit one composite capability call. Evidence describes the returned
//! domain facts, not whether the shell's Result happened to be Ok. A failed
//! call remains a failure even when it has already written partial state.
use crate::ApplicationOutcome;
use crate::CommandFamily;
use crate::EffectId;
use crate::EffectKind;
use crate::EffectMeasure;
use crate::EffectOutput;
use crate::EffectPlan;
use crate::EffectSpec;
use crate::ExpectedOutput;
use crate::Observation;
use crate::ObservationStatus;
use crate::PlanError;
use crate::classify_observations;
use crate::plan_effects;

/// Capability selected by the shell after the operation's effect is admitted.
/// Completion may be synchronous or a future; runtime errors stay in the shell.
pub trait RustOperationPort {
    type Completion;
    fn execute(self) -> Self::Completion;
}

impl<F, T> RustOperationPort for F
where F: FnOnce() -> T
{
    type Completion = T;
    fn execute(self) -> T {
        self()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustOperation {
    ResolveWorkspace,
    CapturePlan,
    ExecuteUnits,
    OpenLocalCache,
    OpenSharedSource,
    ReadSigningKey,
    ServeCache,
    ReadSelfBuildInputs,
    SelfBuild,
    CargoFreeBuild,
    CargoFreeFixedPoint,
    InspectBundle,
    VerifyBundle,
    ExportBundle,
    ReplayBundle,
    ReadTestTarget,
    RealizeTest,
    ReadProfilePackage,
    InspectProfile,
    MutateProfile,
}

impl RustOperation {
    pub fn effect(self) -> (&'static str, EffectKind, CommandFamily) {
        use RustOperation::*;
        match self {
            ResolveWorkspace => ("rust-workspace-directory", EffectKind::ReadEnvironment, CommandFamily::Planning),
            CapturePlan => ("rust-plan-capture", EffectKind::RunProcess, CommandFamily::Planning),
            ExecuteUnits => ("rust-unit-execution", EffectKind::StoreAccess, CommandFamily::Planning),
            OpenLocalCache => ("rust-local-cache-open", EffectKind::StoreAccess, CommandFamily::Planning),
            OpenSharedSource => ("rust-shared-source-open", EffectKind::WriteFiles, CommandFamily::Planning),
            ReadSigningKey => ("rust-cache-signing-key", EffectKind::ReadFiles, CommandFamily::Planning),
            ServeCache => ("rust-cache-serve", EffectKind::StoreAccess, CommandFamily::StoreAdministration),
            ReadSelfBuildInputs => ("self-build-inputs", EffectKind::StoreAccess, CommandFamily::Bootstrap),
            SelfBuild | CargoFreeBuild | CargoFreeFixedPoint => {
                ("rust-self-build", EffectKind::StoreAccess, CommandFamily::Bootstrap)
            }
            InspectBundle => ("cargo-bundle-inspect", EffectKind::ReadFiles, CommandFamily::Planning),
            VerifyBundle => ("cargo-bundle-verify", EffectKind::StoreAccess, CommandFamily::Planning),
            ExportBundle => ("cargo-bundle-export", EffectKind::StoreAccess, CommandFamily::Planning),
            ReplayBundle => ("cargo-bundle-replay", EffectKind::StoreAccess, CommandFamily::Planning),
            ReadTestTarget => ("test-target-read", EffectKind::ReadFiles, CommandFamily::Realization),
            RealizeTest => ("test-realization", EffectKind::StoreAccess, CommandFamily::Realization),
            ReadProfilePackage => ("profile-package-read", EffectKind::ReadFiles, CommandFamily::Project),
            InspectProfile | MutateProfile => ("user-profile-store", EffectKind::StoreAccess, CommandFamily::Project),
        }
    }
}

/// Independent facts retained by the capability adapter. Optional identities
/// are absent when a producer did not supply the required evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustEvidence<'a> {
    Workspace {
        path: &'a [u8],
        absolute: bool,
    },
    Plan {
        receipt_hash: &'a str,
        workspace: &'a str,
        packages: usize,
        declared_packages: usize,
    },
    Units {
        receipt_hash: &'a str,
        execution_succeeded: bool,
        units: usize,
        failed_units: usize,
        missing_artifacts: usize,
    },
    LocalCache {
        reads: bool,
        writes: bool,
        toolchain_digest: &'a str,
    },
    SharedSource {
        directory: &'a str,
        exists: bool,
    },
    SigningKey {
        signer: &'a str,
    },
    Daemon {
        listener_started: bool,
        responses_flushed: usize,
    },
    SelfBuildInputs {
        inventory_digest: Option<&'a str>,
        source_manifest: Option<&'a str>,
        source_state: Option<&'a str>,
        overrides: usize,
        inventory_requested: bool,
        source_requested: bool,
    },
    Binary {
        path: &'a [u8],
        digest: &'a str,
        smoke_exit: Option<i32>,
        smoke_required: bool,
    },
    CargoFree {
        binary_digest: Option<&'a str>,
        cargo_absent: bool,
        child_exit: Option<i32>,
        smoke_exit: Option<i32>,
        failed_units: u64,
    },
    FixedPoint {
        first_digest: Option<&'a str>,
        second_digest: Option<&'a str>,
        first_succeeded: bool,
        second_succeeded: bool,
    },
    BundleManifest {
        bundle_digest: &'a str,
        plan_digest: &'a str,
    },
    Bundle {
        bundle_digest: &'a str,
        source_verified: bool,
        plan_valid: bool,
        cached_results_verified: bool,
        execution_observed: bool,
        planner_rerun: bool,
    },
    TestTarget {
        identity: &'a str,
        input_count: usize,
    },
    TestReport {
        schema_valid: bool,
        identity: &'a str,
    },
    ProfilePackage {
        name: &'a str,
        output_path: &'a str,
    },
    Profile {
        schema_valid: bool,
        operation_matches: bool,
        applied: bool,
        apply_requested: bool,
        inspection: bool,
        removal: bool,
        admission_checked: bool,
        active: Option<u64>,
        previous_active: Option<u64>,
        target: Option<u64>,
        target_retained: bool,
        has_manifest: bool,
    },
    Failed,
}

impl RustEvidence<'_> {
    fn admits(&self, operation: RustOperation) -> bool {
        use RustEvidence::*;
        use RustOperation::*;
        match (operation, self) {
            (ResolveWorkspace, Workspace { path, absolute }) => !path.is_empty() && *absolute,
            (
                CapturePlan,
                Plan {
                    receipt_hash,
                    workspace,
                    packages,
                    declared_packages,
                },
            ) => !receipt_hash.is_empty() && !workspace.is_empty() && packages == declared_packages,
            (
                ExecuteUnits,
                Units {
                    receipt_hash,
                    execution_succeeded,
                    units,
                    failed_units,
                    missing_artifacts,
                },
            ) => {
                !receipt_hash.is_empty()
                    && *execution_succeeded
                    && *units > 0
                    && *failed_units == 0
                    && *missing_artifacts == 0
            }
            (
                OpenLocalCache,
                LocalCache {
                    reads,
                    writes,
                    toolchain_digest,
                },
            ) => (*reads || *writes) && !toolchain_digest.is_empty(),
            (OpenSharedSource, SharedSource { directory, exists }) => !directory.is_empty() && *exists,
            (ReadSigningKey, SigningKey { signer }) => !signer.is_empty(),
            (
                ServeCache,
                Daemon {
                    listener_started,
                    responses_flushed,
                },
            ) => *listener_started && *responses_flushed > 0,
            (
                ReadSelfBuildInputs,
                SelfBuildInputs {
                    inventory_digest,
                    source_manifest,
                    source_state,
                    overrides,
                    inventory_requested,
                    source_requested,
                },
            ) => {
                (!*inventory_requested || inventory_digest.is_some_and(|digest| !digest.is_empty()))
                    && (!*source_requested
                        || (source_manifest.is_some_and(|digest| !digest.is_empty())
                            && source_state.is_some_and(|digest| !digest.is_empty())
                            && *overrides > 0))
            }
            (
                SelfBuild,
                Binary {
                    path,
                    digest,
                    smoke_exit,
                    smoke_required,
                },
            ) => !path.is_empty() && !digest.is_empty() && (!*smoke_required || *smoke_exit == Some(0)),
            (
                CargoFreeBuild,
                CargoFree {
                    binary_digest,
                    cargo_absent,
                    child_exit,
                    smoke_exit,
                    failed_units,
                },
            ) => {
                binary_digest.as_ref().is_some_and(|digest| !digest.is_empty())
                    && *cargo_absent
                    && *child_exit == Some(0)
                    && *smoke_exit == Some(0)
                    && *failed_units == 0
            }
            (
                CargoFreeFixedPoint,
                FixedPoint {
                    first_digest,
                    second_digest,
                    first_succeeded,
                    second_succeeded,
                },
            ) => {
                *first_succeeded
                    && *second_succeeded
                    && first_digest.as_ref().is_some_and(|digest| !digest.is_empty())
                    && first_digest == second_digest
            }
            (
                InspectBundle,
                BundleManifest {
                    bundle_digest,
                    plan_digest,
                },
            ) => !bundle_digest.is_empty() && !plan_digest.is_empty(),
            (
                VerifyBundle | ExportBundle | ReplayBundle,
                Bundle {
                    bundle_digest,
                    source_verified,
                    plan_valid,
                    cached_results_verified,
                    execution_observed,
                    planner_rerun,
                },
            ) => {
                !bundle_digest.is_empty()
                    && *source_verified
                    && *plan_valid
                    && *cached_results_verified
                    && !*planner_rerun
                    && (operation != ReplayBundle || *execution_observed)
            }
            (ReadTestTarget, TestTarget { identity, input_count }) => !identity.is_empty() && *input_count > 0,
            (RealizeTest, TestReport { schema_valid, identity }) => *schema_valid && !identity.is_empty(),
            (ReadProfilePackage, ProfilePackage { name, output_path }) => !name.is_empty() && !output_path.is_empty(),
            (
                InspectProfile | MutateProfile,
                Profile {
                    schema_valid,
                    operation_matches,
                    applied,
                    apply_requested,
                    inspection,
                    removal,
                    admission_checked,
                    active,
                    previous_active,
                    target,
                    target_retained,
                    has_manifest,
                },
            ) => {
                *schema_valid
                    && *operation_matches
                    && *inspection == (operation == InspectProfile)
                    && if *inspection {
                        !*applied && active == previous_active
                    } else {
                        *applied == *apply_requested
                            && target.is_some()
                            && *has_manifest
                            && if *removal {
                                active == previous_active && (!*applied || !*target_retained)
                            } else {
                                *admission_checked && (!*applied || (active == target && *target_retained))
                            }
                    }
            }
            _ => false,
        }
    }
}

pub fn plan_rust_operation(operation: RustOperation) -> Result<EffectPlan, PlanError> {
    let (effect_id, kind, family) = operation.effect();
    plan_effects(family, &[EffectSpec {
        effect_id,
        kind,
        limit: EffectMeasure::Calls(1),
        expected_output: ExpectedOutput::None,
    }])
}

/// The observation reported for one rust operation.
fn rust_observation(operation: RustOperation, is_admitted: bool) -> Observation {
    let (id, kind, _) = operation.effect();
    let status = if is_admitted {
        ObservationStatus::Succeeded
    } else {
        ObservationStatus::Failed
    };
    let diagnostics_code = if is_admitted {
        None
    } else {
        Some("rust-operation-evidence-rejected".into())
    };
    debug_assert_eq!(status == ObservationStatus::Succeeded, is_admitted);
    debug_assert_eq!(diagnostics_code.is_some(), !is_admitted);
    Observation {
        effect_id: EffectId(id.into()),
        kind,
        status,
        output: EffectOutput::None,
        usage: EffectMeasure::Calls(1),
        diagnostics_code,
    }
}

pub fn classify_rust_operation(
    plan: &EffectPlan,
    operation: RustOperation,
    evidence: &RustEvidence<'_>,
) -> ApplicationOutcome {
    let is_admitted = evidence.admits(operation);
    let observation = rust_observation(operation, is_admitted);
    classify_observations(plan, &[observation])
}

#[cfg(test)]
mod tests {
    use super::*;
    fn outcome(operation: RustOperation, evidence: RustEvidence<'_>) -> ApplicationOutcome {
        classify_rust_operation(&plan_rust_operation(operation).unwrap(), operation, &evidence)
    }
    #[test]
    fn successful_status_without_artifacts_is_not_execution_success() {
        assert_eq!(
            outcome(RustOperation::ExecuteUnits, RustEvidence::Units {
                receipt_hash: "receipt",
                execution_succeeded: true,
                units: 1,
                failed_units: 0,
                missing_artifacts: 1
            }),
            ApplicationOutcome::Failed { failed_effect_count: 1 }
        );
    }
    #[test]
    fn mismatching_fixed_point_digests_are_not_success() {
        assert_eq!(
            outcome(RustOperation::CargoFreeFixedPoint, RustEvidence::FixedPoint {
                first_digest: Some("a"),
                second_digest: Some("b"),
                first_succeeded: true,
                second_succeeded: true
            }),
            ApplicationOutcome::Failed { failed_effect_count: 1 }
        );
    }
    #[test]
    fn daemon_listener_without_consumed_request_is_not_success() {
        assert_eq!(
            outcome(RustOperation::ServeCache, RustEvidence::Daemon {
                listener_started: true,
                responses_flushed: 0
            }),
            ApplicationOutcome::Failed { failed_effect_count: 1 }
        );
    }
    #[test]
    fn replay_requires_observed_execution() {
        let evidence = RustEvidence::Bundle {
            bundle_digest: "bundle",
            source_verified: true,
            plan_valid: true,
            cached_results_verified: true,
            execution_observed: false,
            planner_rerun: false,
        };
        assert_eq!(outcome(RustOperation::VerifyBundle, evidence.clone()), ApplicationOutcome::Completed);
        assert_eq!(outcome(RustOperation::ReplayBundle, evidence), ApplicationOutcome::Failed {
            failed_effect_count: 1
        });
    }
    #[test]
    fn cargo_invocation_invalidates_an_otherwise_successful_cargo_free_binary() {
        let evidence = RustEvidence::CargoFree {
            binary_digest: Some("binary"),
            cargo_absent: false,
            child_exit: Some(0),
            smoke_exit: Some(0),
            failed_units: 0,
        };
        assert_eq!(outcome(RustOperation::CargoFreeBuild, evidence), ApplicationOutcome::Failed {
            failed_effect_count: 1
        });
    }

    #[test]
    fn profile_dry_run_does_not_require_a_published_generation() {
        let mut evidence = RustEvidence::Profile {
            schema_valid: true,
            operation_matches: true,
            applied: false,
            apply_requested: false,
            inspection: false,
            removal: false,
            admission_checked: true,
            active: Some(1),
            previous_active: Some(1),
            target: Some(2),
            target_retained: false,
            has_manifest: true,
        };
        assert_eq!(outcome(RustOperation::MutateProfile, evidence.clone()), ApplicationOutcome::Completed);
        if let RustEvidence::Profile {
            applied,
            apply_requested,
            ..
        } = &mut evidence
        {
            *applied = true;
            *apply_requested = true;
        }
        assert_eq!(outcome(RustOperation::MutateProfile, evidence), ApplicationOutcome::Failed {
            failed_effect_count: 1
        });
    }

    #[test]
    fn profile_generation_removal_must_not_switch_the_active_generation() {
        let mut evidence = RustEvidence::Profile {
            schema_valid: true,
            operation_matches: true,
            applied: true,
            apply_requested: true,
            inspection: false,
            removal: true,
            admission_checked: false,
            active: Some(1),
            previous_active: Some(1),
            target: Some(2),
            target_retained: false,
            has_manifest: true,
        };
        assert_eq!(outcome(RustOperation::MutateProfile, evidence.clone()), ApplicationOutcome::Completed);
        if let RustEvidence::Profile { active, .. } = &mut evidence {
            *active = Some(2);
        }
        assert_eq!(outcome(RustOperation::MutateProfile, evidence), ApplicationOutcome::Failed {
            failed_effect_count: 1
        });
    }
}
