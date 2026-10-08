//! Decisions about remote effects, never evidence that an adapter succeeded.

pub const MAX_REMOTE_EFFECTS: u16 = 32;
pub const MAX_EFFECT_BYTES: u64 = 1_099_511_627_776;
pub const MAX_EFFECT_ITEMS: u32 = 4_096;
pub const MAX_EFFECT_TIME_SECS: u64 = 86_400;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Authority {
    Transport,
    AttemptPersistence,
    ExternalBatch,
    Lease,
    InputTransfer,
    OutputTransfer,
    Executor,
    OutputAdmission,
    CredentialVerification,
    Clock,
    RandomIdentifier,
    Telemetry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectKind {
    TransportSend,
    TransportExchange,
    ExternalBatchDispatch,
    AttemptLoad,
    AttemptPersist,
    LeaseReserve,
    LeaseRenew,
    LeaseRelease,
    LeaseQuarantine,
    InputTransfer,
    OutputTransfer,
    ExecutorLaunch,
    OutputAdmission,
    CredentialVerify,
    ClockObserve,
    RandomIdentifier,
    TelemetryPublish,
}

impl EffectKind {
    pub const fn authority(self) -> Authority {
        match self {
            Self::TransportSend | Self::TransportExchange => Authority::Transport,
            Self::ExternalBatchDispatch => Authority::ExternalBatch,
            Self::AttemptLoad | Self::AttemptPersist => Authority::AttemptPersistence,
            Self::LeaseReserve | Self::LeaseRenew | Self::LeaseRelease | Self::LeaseQuarantine => Authority::Lease,
            Self::InputTransfer => Authority::InputTransfer,
            Self::OutputTransfer => Authority::OutputTransfer,
            Self::ExecutorLaunch => Authority::Executor,
            Self::OutputAdmission => Authority::OutputAdmission,
            Self::CredentialVerify => Authority::CredentialVerification,
            Self::ClockObserve => Authority::Clock,
            Self::RandomIdentifier => Authority::RandomIdentifier,
            Self::TelemetryPublish => Authority::Telemetry,
        }
    }
}

/// One fenced assignment can have independent local actors in separate
/// processes. Their effect sequence numbers never alias across that bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectActor {
    Unspecified,
    Client,
    ClientProtocol,
    Worker,
    FailureObservability,
    TelemetryExport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectId<'a> {
    pub request_id: &'a str,
    pub actor: EffectActor,
    pub binding: Option<RemoteEffectBinding<'a>>,
    pub sequence: u16,
}
/// Durable identity shared by effects within one local process. The client and
/// worker each construct their own session from the same fenced assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteEffectBinding<'a> {
    pub job_id: &'a crate::attempt::RemoteJobId,
    pub attempt_id: &'a crate::attempt::RemoteAttemptId,
    pub fence_generation: crate::attempt::RemoteFenceGeneration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectLimits {
    pub bytes_max: u64,
    pub items_max: u32,
    pub time_secs_max: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Effect<'a> {
    pub id: EffectId<'a>,
    pub kind: EffectKind,
    pub authority: Authority,
    pub limits: EffectLimits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedEffect {
    TransportSent {
        bytes: u64,
    },
    TransportExchanged {
        frames: u32,
    },
    /// A provider response was validated and observed, not a completed build.
    ExternalBatchObserved {
        state: crate::external_batch::BatchState,
    },
    AttemptLoaded,
    AttemptPersisted,
    LeaseReserved,
    LeaseRenewed,
    LeaseReleased,
    LeaseQuarantined,
    InputsTransferred {
        bytes: u64,
    },
    OutputsTransferred {
        bytes: u64,
    },
    ExecutorCompleted {
        outputs: u32,
    },
    /// Completed output count is a lower bound; the failing output may have
    /// been partly persisted before its store operation returned an error.
    OutputsPartiallyAdmitted {
        outputs: u32,
        current_output_may_be_durable: bool,
    },
    OutputsAdmitted {
        outputs: u32,
    },
    CredentialVerified,
    ClockObserved,
    IdentifierGenerated {
        bytes: u64,
    },
    TelemetryPublished {
        bytes: u64,
    },
    TelemetryEventsPublished {
        events: u32,
    },
}

impl ObservedEffect {
    pub const fn kind(self) -> EffectKind {
        match self {
            Self::TransportSent { .. } => EffectKind::TransportSend,
            Self::TransportExchanged { .. } => EffectKind::TransportExchange,
            Self::ExternalBatchObserved { .. } => EffectKind::ExternalBatchDispatch,
            Self::AttemptLoaded => EffectKind::AttemptLoad,
            Self::AttemptPersisted => EffectKind::AttemptPersist,
            Self::LeaseReserved => EffectKind::LeaseReserve,
            Self::LeaseRenewed => EffectKind::LeaseRenew,
            Self::LeaseReleased => EffectKind::LeaseRelease,
            Self::LeaseQuarantined => EffectKind::LeaseQuarantine,
            Self::InputsTransferred { .. } => EffectKind::InputTransfer,
            Self::OutputsTransferred { .. } => EffectKind::OutputTransfer,
            Self::ExecutorCompleted { .. } => EffectKind::ExecutorLaunch,
            Self::OutputsAdmitted { .. } | Self::OutputsPartiallyAdmitted { .. } => EffectKind::OutputAdmission,
            Self::CredentialVerified => EffectKind::CredentialVerify,
            Self::ClockObserved => EffectKind::ClockObserve,
            Self::IdentifierGenerated { .. } => EffectKind::RandomIdentifier,
            Self::TelemetryPublished { .. } | Self::TelemetryEventsPublished { .. } => EffectKind::TelemetryPublish,
        }
    }

    const fn usage(self) -> (u64, u32) {
        match self {
            Self::TransportSent { bytes }
            | Self::InputsTransferred { bytes }
            | Self::OutputsTransferred { bytes }
            | Self::IdentifierGenerated { bytes }
            | Self::TelemetryPublished { bytes } => (bytes, 1),
            Self::TransportExchanged { frames } => (0, frames),
            Self::ExecutorCompleted { outputs }
            | Self::OutputsAdmitted { outputs }
            | Self::OutputsPartiallyAdmitted { outputs, .. } => (0, outputs),
            Self::TelemetryEventsPublished { events } => (0, events),
            _ => (0, 1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Observation<'a> {
    Succeeded { id: EffectId<'a>, effect: ObservedEffect },
    Failed { id: EffectId<'a>, authority: Authority },
    PartiallyFailed { id: EffectId<'a>, effect: ObservedEffect },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectBlocker {
    RequestIdEmpty,
    EffectCountExceeded,
    EffectLimitExceeded,
    PendingEffect,
    NoPendingEffect,
    SessionFailed,
    WrongEffectId,
    WrongAuthority,
    WrongObservationKind,
    ObservedLimitExceeded,
    AttemptBindingMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectEvent<'a> {
    Completed(EffectId<'a>),
    Failed(EffectId<'a>),
    PartiallyFailed {
        id: EffectId<'a>,
        outputs: u32,
        current_output_may_be_durable: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectSession<'a> {
    request_id: &'a str,
    actor: EffectActor,
    binding: Option<RemoteEffectBinding<'a>>,
    next_sequence: u16,
    pending: Option<Effect<'a>>,
    failed: bool,
}

impl<'a> EffectSession<'a> {
    pub fn new(request_id: &'a str) -> Result<Self, EffectBlocker> {
        if request_id.is_empty() {
            return Err(EffectBlocker::RequestIdEmpty);
        }
        Ok(Self {
            request_id,
            actor: EffectActor::Unspecified,
            next_sequence: 0,
            pending: None,
            failed: false,
            binding: None,
        })
    }

    pub fn for_attempt(request_id: &'a str, binding: RemoteEffectBinding<'a>) -> Result<Self, EffectBlocker> {
        Self::for_actor_attempt(request_id, binding, EffectActor::Unspecified)
    }

    pub fn for_actor_attempt(
        request_id: &'a str,
        binding: RemoteEffectBinding<'a>,
        actor: EffectActor,
    ) -> Result<Self, EffectBlocker> {
        let mut session = Self::new(request_id)?;
        session.binding = Some(binding);
        session.actor = actor;
        Ok(session)
    }

    pub fn require_attempt(
        &self,
        job_id: &crate::attempt::RemoteJobId,
        attempt_id: &crate::attempt::RemoteAttemptId,
        fence_generation: crate::attempt::RemoteFenceGeneration,
    ) -> Result<(), EffectBlocker> {
        match self.binding {
            Some(binding)
                if binding.job_id == job_id
                    && binding.attempt_id == attempt_id
                    && binding.fence_generation == fence_generation =>
            {
                Ok(())
            }
            _ => Err(EffectBlocker::AttemptBindingMismatch),
        }
    }

    pub fn binding(&self) -> Option<RemoteEffectBinding<'a>> {
        self.binding
    }

    pub fn request_id(&self) -> &str {
        self.request_id
    }

    /// There is at most one outstanding effect. A dependent plan is impossible
    /// until the matching typed observation has been admitted.
    // r[impl remote_builds.remote_effect_plans]
    pub fn plan(&mut self, kind: EffectKind, limits: EffectLimits) -> Result<Effect<'a>, EffectBlocker> {
        if self.failed {
            return Err(EffectBlocker::SessionFailed);
        }
        if self.pending.is_some() {
            return Err(EffectBlocker::PendingEffect);
        }
        if self.next_sequence >= MAX_REMOTE_EFFECTS {
            return Err(EffectBlocker::EffectCountExceeded);
        }
        if limits.bytes_max > MAX_EFFECT_BYTES
            || limits.items_max > MAX_EFFECT_ITEMS
            || limits.time_secs_max > MAX_EFFECT_TIME_SECS
        {
            return Err(EffectBlocker::EffectLimitExceeded);
        }
        let effect = Effect {
            id: EffectId {
                request_id: self.request_id,
                actor: self.actor,
                binding: self.binding,
                sequence: self.next_sequence,
            },
            kind,
            authority: kind.authority(),
            limits,
        };
        self.pending = Some(effect);
        Ok(effect)
    }

    // r[impl remote_builds.remote_effect_plans]
    pub fn observe(&mut self, observation: Observation<'_>) -> Result<EffectEvent<'a>, EffectBlocker> {
        let pending = self.pending.ok_or(EffectBlocker::NoPendingEffect)?;
        let (id, authority) = match observation {
            Observation::Succeeded { id, effect } | Observation::PartiallyFailed { id, effect } => {
                (id, effect.kind().authority())
            }
            Observation::Failed { id, authority } => (id, authority),
        };
        if id != pending.id {
            return Err(EffectBlocker::WrongEffectId);
        }
        if authority != pending.authority {
            return Err(EffectBlocker::WrongAuthority);
        }
        if let Observation::Succeeded { effect, .. } | Observation::PartiallyFailed { effect, .. } = observation {
            if matches!(observation, Observation::PartiallyFailed { .. })
                != matches!(effect, ObservedEffect::OutputsPartiallyAdmitted { .. })
            {
                return Err(EffectBlocker::WrongObservationKind);
            }
            if effect.kind() != pending.kind {
                return Err(EffectBlocker::WrongObservationKind);
            }
            let (bytes, items) = effect.usage();
            if bytes > pending.limits.bytes_max || items > pending.limits.items_max {
                return Err(EffectBlocker::ObservedLimitExceeded);
            }
        }
        self.pending = None;
        self.next_sequence += 1;
        match observation {
            Observation::Succeeded { .. } => Ok(EffectEvent::Completed(pending.id)),
            Observation::Failed { .. } => {
                self.failed = true;
                Ok(EffectEvent::Failed(pending.id))
            }
            Observation::PartiallyFailed {
                effect:
                    ObservedEffect::OutputsPartiallyAdmitted {
                        outputs,
                        current_output_may_be_durable,
                    },
                ..
            } => {
                self.failed = true;
                Ok(EffectEvent::PartiallyFailed {
                    id: pending.id,
                    outputs,
                    current_output_may_be_durable,
                })
            }
            Observation::PartiallyFailed { .. } => {
                unreachable!("partial kind was validated before applying observation")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependent_effect_requires_matching_bounded_observation() {
        let mut session = EffectSession::new("request-1").unwrap();
        let limits = EffectLimits {
            bytes_max: 64,
            items_max: 1,
            time_secs_max: 60,
        };
        let transport = session.plan(EffectKind::TransportSend, limits).unwrap();
        assert_eq!(session.plan(EffectKind::ExecutorLaunch, limits), Err(EffectBlocker::PendingEffect));
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: EffectId {
                    sequence: 1,
                    ..transport.id
                },
                effect: ObservedEffect::TransportSent { bytes: 10 },
            }),
            Err(EffectBlocker::WrongEffectId)
        );
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: transport.id,
                effect: ObservedEffect::TransportSent { bytes: 65 },
            }),
            Err(EffectBlocker::ObservedLimitExceeded)
        );
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: transport.id,
                effect: ObservedEffect::TransportSent { bytes: 64 },
            }),
            Ok(EffectEvent::Completed(transport.id))
        );
        let executor = session.plan(EffectKind::ExecutorLaunch, limits).unwrap();
        assert_eq!(executor.id.sequence, 1);
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: executor.id,
                effect: ObservedEffect::OutputsAdmitted { outputs: 1 },
            }),
            Err(EffectBlocker::WrongAuthority)
        );
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: executor.id,
                effect: ObservedEffect::ExecutorCompleted { outputs: 1 },
            }),
            Ok(EffectEvent::Completed(executor.id))
        );
    }

    #[test]
    fn failed_adapter_observation_is_terminal_and_cannot_claim_success() {
        let mut session = EffectSession::new("request-2").unwrap();
        let limits = EffectLimits {
            bytes_max: 0,
            items_max: 1,
            time_secs_max: 60,
        };
        let effect = session.plan(EffectKind::CredentialVerify, limits).unwrap();
        assert_eq!(
            session.observe(Observation::Failed {
                id: effect.id,
                authority: Authority::CredentialVerification,
            }),
            Ok(EffectEvent::Failed(effect.id))
        );
        assert_eq!(session.plan(EffectKind::ExecutorLaunch, limits), Err(EffectBlocker::SessionFailed));
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: effect.id,
                effect: ObservedEffect::CredentialVerified,
            }),
            Err(EffectBlocker::NoPendingEffect)
        );
    }
    #[test]
    fn lease_renewal_and_output_transfer_need_exact_observations() {
        let mut session = EffectSession::new("attempt-3").unwrap();
        let limits = EffectLimits {
            bytes_max: 4,
            items_max: 1,
            time_secs_max: 0,
        };
        let renew = session.plan(EffectKind::LeaseRenew, limits).unwrap();
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: renew.id,
                effect: ObservedEffect::LeaseReserved
            }),
            Err(EffectBlocker::WrongObservationKind),
        );
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: renew.id,
                effect: ObservedEffect::LeaseRenewed
            }),
            Ok(EffectEvent::Completed(renew.id)),
        );
        let transfer = session.plan(EffectKind::OutputTransfer, limits).unwrap();
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: transfer.id,
                effect: ObservedEffect::OutputsTransferred { bytes: 5 }
            }),
            Err(EffectBlocker::ObservedLimitExceeded),
        );
        assert_eq!(session.plan(EffectKind::TelemetryPublish, limits), Err(EffectBlocker::PendingEffect));
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: transfer.id,
                effect: ObservedEffect::OutputsTransferred { bytes: 4 }
            }),
            Ok(EffectEvent::Completed(transfer.id)),
        );
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: transfer.id,
                effect: ObservedEffect::OutputsTransferred { bytes: 4 }
            }),
            Err(EffectBlocker::NoPendingEffect),
        );
    }
    #[test]
    fn fenced_session_rejects_other_job_attempt_and_generation_before_effect_planning() {
        use crate::attempt::RemoteAttemptId;
        use crate::attempt::RemoteFenceGeneration;
        use crate::attempt::RemoteJobId;

        let job_id = RemoteJobId::new("job-one").unwrap();
        let other_job = RemoteJobId::new("job-two").unwrap();
        let attempt_id = RemoteAttemptId::new("attempt-one").unwrap();
        let other_attempt = RemoteAttemptId::new("attempt-two").unwrap();
        let generation = RemoteFenceGeneration::new(3).unwrap();
        let wrong_generation = RemoteFenceGeneration::new(4).unwrap();
        let mut session = EffectSession::for_attempt("request-one", RemoteEffectBinding {
            job_id: &job_id,
            attempt_id: &attempt_id,
            fence_generation: generation,
        })
        .unwrap();
        assert_eq!(
            session.require_attempt(&other_job, &attempt_id, generation),
            Err(EffectBlocker::AttemptBindingMismatch)
        );
        assert_eq!(
            session.require_attempt(&job_id, &other_attempt, generation),
            Err(EffectBlocker::AttemptBindingMismatch)
        );
        assert_eq!(
            session.require_attempt(&job_id, &attempt_id, wrong_generation),
            Err(EffectBlocker::AttemptBindingMismatch)
        );
        assert_eq!(session.require_attempt(&job_id, &attempt_id, generation), Ok(()));
        let effect = session
            .plan(EffectKind::TransportSend, EffectLimits {
                bytes_max: 4,
                items_max: 1,
                time_secs_max: 0,
            })
            .unwrap();
        assert_eq!(effect.id.sequence, 0);
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: effect.id,
                effect: ObservedEffect::TransportSent { bytes: 4 },
            }),
            Ok(EffectEvent::Completed(effect.id))
        );
    }

    #[test]
    fn partially_durable_store_failure_cannot_complete_or_persist_attempt() {
        let mut session = EffectSession::new("request-two").unwrap();
        let limits = EffectLimits {
            bytes_max: 0,
            items_max: 2,
            time_secs_max: 0,
        };
        let admission = session.plan(EffectKind::OutputAdmission, limits).unwrap();
        let partial = ObservedEffect::OutputsPartiallyAdmitted {
            outputs: 1,
            current_output_may_be_durable: true,
        };
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: admission.id,
                effect: partial
            }),
            Err(EffectBlocker::WrongObservationKind),
        );
        assert_eq!(session.plan(EffectKind::AttemptPersist, limits), Err(EffectBlocker::PendingEffect));
        assert_eq!(
            session.observe(Observation::PartiallyFailed {
                id: admission.id,
                effect: ObservedEffect::OutputsPartiallyAdmitted {
                    outputs: 3,
                    current_output_may_be_durable: true,
                },
            }),
            Err(EffectBlocker::ObservedLimitExceeded),
        );
        assert_eq!(
            session.observe(Observation::PartiallyFailed {
                id: admission.id,
                effect: partial
            }),
            Ok(EffectEvent::PartiallyFailed {
                id: admission.id,
                outputs: 1,
                current_output_may_be_durable: true,
            }),
        );
        assert_eq!(session.plan(EffectKind::AttemptPersist, limits), Err(EffectBlocker::SessionFailed));
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: admission.id,
                effect: ObservedEffect::OutputsAdmitted { outputs: 2 },
            }),
            Err(EffectBlocker::NoPendingEffect),
        );
    }
    #[test]
    fn client_exchange_observation_requires_exact_actor_fence_and_frame_limit() {
        use crate::attempt::{RemoteAttemptId, RemoteFenceGeneration, RemoteJobId};

        let job = RemoteJobId::new("job-exchange").unwrap();
        let attempt = RemoteAttemptId::new("attempt-exchange").unwrap();
        let generation = RemoteFenceGeneration::new(5).unwrap();
        let mut session = EffectSession::for_actor_attempt(
            "request-exchange",
            RemoteEffectBinding {
                job_id: &job,
                attempt_id: &attempt,
                fence_generation: generation,
            },
            EffectActor::Client,
        )
        .unwrap();
        let effect = session
            .plan(EffectKind::TransportExchange, EffectLimits {
                bytes_max: 0,
                items_max: 3,
                time_secs_max: 1,
            })
            .unwrap();
        let mut wrong_actor = effect.id;
        wrong_actor.actor = EffectActor::Worker;
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: wrong_actor,
                effect: ObservedEffect::TransportExchanged { frames: 2 },
            }),
            Err(EffectBlocker::WrongEffectId),
        );
        let mut wrong_fence = effect.id;
        wrong_fence.binding = Some(RemoteEffectBinding {
            job_id: &job,
            attempt_id: &attempt,
            fence_generation: RemoteFenceGeneration::new(6).unwrap(),
        });
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: wrong_fence,
                effect: ObservedEffect::TransportExchanged { frames: 2 },
            }),
            Err(EffectBlocker::WrongEffectId),
        );
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: effect.id,
                effect: ObservedEffect::TransportExchanged { frames: 4 },
            }),
            Err(EffectBlocker::ObservedLimitExceeded),
        );
        assert_eq!(
            session.observe(Observation::Succeeded {
                id: effect.id,
                effect: ObservedEffect::TransportExchanged { frames: 3 },
            }),
            Ok(EffectEvent::Completed(effect.id)),
        );
    }
}
