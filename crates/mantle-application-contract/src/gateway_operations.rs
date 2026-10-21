//! Gateway CLI operation boundaries. Basalt admission, signed-store verification,
//! journal reconciliation and quota policy remain owned by the existing gateway.
//! This module declares host authorities and checks independently returned facts;
//! it neither mints authority nor treats a handler's `Ok(())` as evidence.
use alloc::string::String;
use alloc::vec::Vec;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayOperation {
    Read,
    NixImportOnce,
    NixStoreReadOnce,
    NixStoreReadBinding,
    NixUploadBinding,
    QuotaFaultCampaign,
    ProvisionQuota,
    Binding,
    Call,
    Serve,
    NixOnce,
}

impl GatewayOperation {
    /// Each entry is one invocation of an owning composite port with this
    /// authority, not a claimed bound on its internal syscalls or protocol frames.
    fn authorities(self) -> &'static [(&'static str, EffectKind)] {
        use EffectKind::*;
        match self {
            Self::NixStoreReadBinding => &[],
            Self::Binding | Self::NixUploadBinding => &[("gateway-binding-input", ReadFiles)],
            Self::ProvisionQuota | Self::QuotaFaultCampaign => &[
                ("gateway-quota-directory", WriteFiles),
                ("gateway-quota-ledger", StoreAccess),
            ],
            Self::Call => &[
                ("gateway-call-envelope", ReadFiles),
                ("gateway-call-peer", InterprocessCommunication),
            ],
            Self::Read | Self::NixImportOnce | Self::NixStoreReadOnce => &[
                ("gateway-private-input", ReadFiles),
                ("gateway-private-state", WriteFiles),
                ("gateway-private-store", StoreAccess),
                ("gateway-private-peer", InterprocessCommunication),
                ("gateway-private-time", ReadClock),
            ],
            Self::Serve | Self::NixOnce => &[
                ("gateway-service-input", ReadFiles),
                ("gateway-service-state", WriteFiles),
                ("gateway-service-store", StoreAccess),
                ("gateway-service-peer", InterprocessCommunication),
                // Child/network execution retains the nested remote execution
                // family's admission; idle serving does not observe such work.
                ("gateway-service-time", ReadClock),
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayOperationPlan {
    pub operation: GatewayOperation,
    /// Pure store-binding calculation has no IO authority, including no fake
    /// read effect. Rendering is performed only after classification.
    pub effects: Option<EffectPlan>,
}

pub fn plan_gateway_operation(operation: GatewayOperation) -> Result<GatewayOperationPlan, PlanError> {
    let specs: Vec<_> = operation
        .authorities()
        .iter()
        .map(|(id, kind)| EffectSpec {
            effect_id: id,
            kind: *kind,
            limit: EffectMeasure::Calls(1),
            expected_output: ExpectedOutput::None,
        })
        .collect();
    let effects = if specs.is_empty() {
        None
    } else {
        Some(crate::plan_effects(CommandFamily::RemoteExecution, &specs)?)
    };
    Ok(GatewayOperationPlan { operation, effects })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayOperationEvidence {
    PureBinding,
    BindingInput {
        bytes_read: usize,
        maximum: usize,
    },
    Call {
        envelope_bytes: usize,
        response_bytes: usize,
        version: Option<u64>,
        has_response: bool,
        has_error: bool,
    },
    /// Inode comes from the actual bound socket; shutdown comes from the signal
    /// branch, not from an unconditional stop flag set after a worker returns.
    Service {
        socket_inode: u64,
        shutdown_requested: bool,
        workers_started: usize,
        workers_joined: usize,
    },
    Import {
        declared_bytes: u64,
        verified_bytes: u64,
        admitted_bytes: u64,
        missing_paths: usize,
    },
    StoreRead {
        authorized_writes: u64,
        response_bytes: u64,
    },
    QuotaProvision {
        ledger_schema: String,
        expected_schema: &'static str,
    },
    QuotaFixture {
        expected_steps: usize,
        observed_steps: usize,
        expected_errors: usize,
        observed_errors: usize,
        ledger_entries: usize,
        project_charged: u64,
        account_charged: u64,
        project_reserved: u64,
        account_reserved: u64,
    },
    Build {
        requests: u64,
        completed: u64,
        failed: u64,
        missing_result_evidence: u64,
        missing_outputs: usize,
    },
}

pub struct GatewayOperationResult {
    pub evidence: GatewayOperationEvidence,
    pub output: Option<String>,
}

/// The request DTO and domain error belong to the host. Planning precedes port
/// entry; classification precedes rendering. No host error is erased here.
pub trait GatewayOperationPort {
    type Request;
    type Error;
    fn execute(
        &mut self,
        plan: &GatewayOperationPlan,
        request: Self::Request,
    ) -> Result<GatewayOperationResult, Self::Error>;
}

pub fn classify_gateway_operation(plan: &GatewayOperationPlan, facts: &GatewayOperationEvidence) -> ApplicationOutcome {
    use GatewayOperation as Op;
    use GatewayOperationEvidence as Facts;
    let supported = match (plan.operation, facts) {
        (Op::NixStoreReadBinding, Facts::PureBinding) => true,
        (Op::Binding | Op::NixUploadBinding, Facts::BindingInput { bytes_read, maximum }) => {
            *bytes_read > 0 && bytes_read <= maximum
        }
        (
            Op::Call,
            Facts::Call {
                envelope_bytes,
                response_bytes,
                version,
                has_response,
                has_error,
            },
        ) => {
            *envelope_bytes > 0
                && *envelope_bytes <= 128 * 1024
                && *response_bytes > 0
                && *response_bytes <= 1024 * 1024
                && *version == Some(1)
                && *has_response
                && !has_error
        }
        (
            Op::Read | Op::Serve,
            Facts::Service {
                socket_inode,
                shutdown_requested,
                workers_started,
                workers_joined,
            },
        ) => *socket_inode != 0 && *shutdown_requested && workers_started == workers_joined,
        (
            Op::NixImportOnce,
            Facts::Import {
                declared_bytes,
                verified_bytes,
                admitted_bytes,
                missing_paths,
            },
        ) => {
            *declared_bytes > 0
                && declared_bytes == verified_bytes
                && verified_bytes <= admitted_bytes
                && *missing_paths == 0
        }
        (
            Op::NixStoreReadOnce,
            Facts::StoreRead {
                authorized_writes,
                response_bytes,
            },
        ) => *authorized_writes > 0 && *response_bytes > 0,
        (
            Op::ProvisionQuota,
            Facts::QuotaProvision {
                ledger_schema,
                expected_schema,
            },
        ) => !expected_schema.is_empty() && ledger_schema.as_str() == *expected_schema,
        (
            Op::QuotaFaultCampaign,
            Facts::QuotaFixture {
                expected_steps,
                observed_steps,
                expected_errors,
                observed_errors,
                ledger_entries,
                project_charged,
                account_charged,
                project_reserved,
                account_reserved,
            },
        ) => {
            *expected_steps > 0
                && expected_steps == observed_steps
                && *expected_errors == 4
                && expected_errors == observed_errors
                && *ledger_entries == 1
                && *project_charged == 1
                && *account_charged == 1
                && *project_reserved == 0
                && *account_reserved == 0
        }
        (
            Op::NixOnce,
            Facts::Build {
                requests,
                completed,
                failed,
                missing_result_evidence,
                missing_outputs,
            },
        ) => {
            *requests > 0
                && requests == completed
                && *failed == 0
                && *missing_result_evidence == 0
                && *missing_outputs == 0
        }
        _ => false,
    };
    let Some(effects) = &plan.effects else {
        return if supported && plan.operation == Op::NixStoreReadBinding {
            ApplicationOutcome::Completed
        } else {
            ApplicationOutcome::Contradicted { effect_count: 1 }
        };
    };
    let observations: Vec<_> = plan
        .operation
        .authorities()
        .iter()
        .map(|(id, kind)| Observation {
            effect_id: EffectId(String::from(*id)),
            kind: *kind,
            status: if supported {
                ObservationStatus::Succeeded
            } else {
                ObservationStatus::Failed
            },
            output: EffectOutput::None,
            usage: EffectMeasure::Calls(1),
            diagnostics_code: if supported {
                None
            } else {
                Some(String::from("gateway-operation-evidence-contradicted"))
            },
        })
        .collect();
    crate::classify_observations(effects, &observations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peer_error_and_unversioned_json_are_not_success() {
        let plan = plan_gateway_operation(GatewayOperation::Call).unwrap();
        for (version, has_response, has_error) in [(Some(1), false, true), (None, true, false), (Some(1), true, true)] {
            let facts = GatewayOperationEvidence::Call {
                envelope_bytes: 100,
                response_bytes: 100,
                version,
                has_response,
                has_error,
            };
            assert!(matches!(classify_gateway_operation(&plan, &facts), ApplicationOutcome::Failed { .. }));
        }
    }

    #[test]
    fn protocol_disconnect_or_rejected_build_is_not_a_completed_build() {
        let plan = plan_gateway_operation(GatewayOperation::NixOnce).unwrap();
        for (requests, completed, failed, missing_result_evidence, missing_outputs) in
            [(0, 0, 0, 0, 0), (1, 0, 1, 0, 0), (1, 1, 0, 1, 0), (1, 1, 0, 0, 1)]
        {
            let facts = GatewayOperationEvidence::Build {
                requests,
                completed,
                failed,
                missing_result_evidence,
                missing_outputs,
            };
            assert!(matches!(classify_gateway_operation(&plan, &facts), ApplicationOutcome::Failed { .. }));
        }
        let facts = GatewayOperationEvidence::Build {
            requests: 1,
            completed: 1,
            failed: 0,
            missing_result_evidence: 0,
            missing_outputs: 0,
        };
        assert_eq!(classify_gateway_operation(&plan, &facts), ApplicationOutcome::Completed);
    }

    #[test]
    fn pure_binding_is_not_evidence_of_a_store_read() {
        let plan = plan_gateway_operation(GatewayOperation::NixStoreReadOnce).unwrap();
        assert!(matches!(
            classify_gateway_operation(&plan, &GatewayOperationEvidence::PureBinding),
            ApplicationOutcome::Failed { .. }
        ));
        let pure = plan_gateway_operation(GatewayOperation::NixStoreReadBinding).unwrap();
        assert!(pure.effects.is_none());
        assert_eq!(
            classify_gateway_operation(&pure, &GatewayOperationEvidence::PureBinding),
            ApplicationOutcome::Completed
        );
    }

    #[test]
    fn import_requires_exact_verified_bytes_and_retained_path() {
        let plan = plan_gateway_operation(GatewayOperation::NixImportOnce).unwrap();
        for (verified_bytes, admitted_bytes, missing_paths) in [(9, 10, 0), (10, 9, 0), (10, 10, 1)] {
            let facts = GatewayOperationEvidence::Import {
                declared_bytes: 10,
                verified_bytes,
                admitted_bytes,
                missing_paths,
            };
            assert!(matches!(classify_gateway_operation(&plan, &facts), ApplicationOutcome::Failed { .. }));
        }
        let facts = GatewayOperationEvidence::Import {
            declared_bytes: 10,
            verified_bytes: 10,
            admitted_bytes: 10,
            missing_paths: 0,
        };
        assert_eq!(classify_gateway_operation(&plan, &facts), ApplicationOutcome::Completed);
    }

    #[test]
    fn graceful_idle_service_is_valid_but_early_worker_exit_is_not() {
        let plan = plan_gateway_operation(GatewayOperation::Read).unwrap();
        let idle = GatewayOperationEvidence::Service {
            socket_inode: 42,
            shutdown_requested: true,
            workers_started: 1,
            workers_joined: 1,
        };
        assert_eq!(classify_gateway_operation(&plan, &idle), ApplicationOutcome::Completed);
        let early = GatewayOperationEvidence::Service {
            socket_inode: 42,
            shutdown_requested: false,
            workers_started: 1,
            workers_joined: 1,
        };
        assert!(matches!(classify_gateway_operation(&plan, &early), ApplicationOutcome::Failed { .. }));
        let unjoined = GatewayOperationEvidence::Service {
            socket_inode: 42,
            shutdown_requested: true,
            workers_started: 1,
            workers_joined: 0,
        };
        assert!(matches!(classify_gateway_operation(&plan, &unjoined), ApplicationOutcome::Failed { .. }));
    }
}
