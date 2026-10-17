use alloc::format;
use alloc::string::String;

/// Admitted direction after an inbound transport adapter has decoded one frame.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    ClientToBuilder,
    BuilderToClient,
}

impl Direction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ClientToBuilder => "client-to-builder",
            Self::BuilderToClient => "builder-to-client",
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Open,
    AwaitAuth,
    AwaitAuthOk,
    AwaitBuildRequest,
    AwaitInputManifest,
    AwaitMissingInputs,
    AwaitInputUpload,
    AwaitQueueAdmission,
    Queued,
    Building,
    AwaitOutputTransfer,
    Done,
    Failed,
}

impl Phase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::AwaitAuth => "await-auth",
            Self::AwaitAuthOk => "await-auth-ok",
            Self::AwaitBuildRequest => "await-build-request",
            Self::AwaitInputManifest => "await-input-manifest",
            Self::AwaitMissingInputs => "await-missing-inputs",
            Self::AwaitInputUpload => "await-input-upload",
            Self::AwaitQueueAdmission => "await-queue-admission",
            Self::Queued => "queued",
            Self::Building => "building",
            Self::AwaitOutputTransfer => "await-output-transfer",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

/// The frame's structural kind, never its untrusted payload or its serialized bytes.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind {
    Hello,
    TraceContext,
    TraceContextAck,
    AuthTicket,
    AuthOk,
    BuildRequest,
    InputManifest,
    MissingInputs,
    InputUpload,
    TransferManifest,
    TransferDemand,
    TransferCredit,
    TransferAcknowledgement,
    TransferComplete,
    BuildQueued,
    BuildStarted,
    BuildFinished,
    OutputTransferArtifact,
    OutputTransferDone,
    Done,
    Error,
}

impl FrameKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hello => "hello",
            Self::TraceContext => "trace-context",
            Self::TraceContextAck => "trace-context-ack",
            Self::AuthTicket => "auth-ticket",
            Self::AuthOk => "auth-ok",
            Self::BuildRequest => "build-request",
            Self::InputManifest => "input-manifest",
            Self::MissingInputs => "missing-inputs",
            Self::InputUpload => "input-upload",
            Self::TransferManifest => "transfer-manifest",
            Self::TransferDemand => "transfer-demand",
            Self::TransferCredit => "transfer-credit",
            Self::TransferAcknowledgement => "transfer-acknowledgement",
            Self::TransferComplete => "transfer-complete",
            Self::BuildQueued => "build-queued",
            Self::BuildStarted => "build-started",
            Self::BuildFinished => "build-finished",
            Self::OutputTransferArtifact => "output-transfer-artifact",
            Self::OutputTransferDone => "output-transfer-done",
            Self::Done => "done",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionError {
    pub phase: Phase,
    pub direction: Direction,
    pub frame: FrameKind,
}

impl TransitionError {
    /// Compatibility diagnostic belongs to the outward protocol adapter.
    pub fn diagnostic(self) -> String {
        format!(
            "unexpected-remote-frame phase={} direction={} frame={}",
            self.phase.as_str(),
            self.direction.as_str(),
            self.frame.as_str()
        )
    }
}

/// Match the accepted protocol matrix exactly. `Error` is terminal from every
/// state and direction; transfer data-plane frames remain out-of-band as before.
/// This decision never claims that a frame was transmitted or an effect succeeded.
// r[impl remote_builds.hexagonal_core]
pub const fn transition(phase: Phase, direction: Direction, frame: FrameKind) -> Result<Phase, TransitionError> {
    use Direction::BuilderToClient as B;
    use Direction::ClientToBuilder as C;
    use FrameKind as F;
    use Phase as P;
    let next = match (phase, direction, frame) {
        (_, _, F::Error) => P::Failed,
        (P::Open, C, F::Hello) => P::AwaitAuth,
        (P::AwaitAuth, C, F::TraceContext) => P::AwaitAuth,
        (P::AwaitAuth, C, F::AuthTicket) => P::AwaitAuthOk,
        (P::AwaitAuthOk, B, F::AuthOk) => P::AwaitBuildRequest,
        (P::AwaitBuildRequest, B, F::TraceContextAck) => P::AwaitBuildRequest,
        (P::AwaitBuildRequest, C, F::BuildRequest) => P::AwaitInputManifest,
        (P::AwaitInputManifest, C, F::InputManifest) => P::AwaitMissingInputs,
        (P::AwaitMissingInputs, B, F::MissingInputs) => P::AwaitInputUpload,
        (P::AwaitInputUpload, C, F::InputUpload) => P::AwaitQueueAdmission,
        (P::AwaitQueueAdmission, B, F::BuildQueued) => P::Queued,
        (P::Queued, B, F::BuildStarted) => P::Building,
        (P::Building, B, F::BuildFinished) => P::AwaitOutputTransfer,
        (P::AwaitOutputTransfer, B, F::OutputTransferArtifact) => P::AwaitOutputTransfer,
        (P::AwaitOutputTransfer, B, F::OutputTransferDone) => P::Done,
        (P::Done, B, F::Done) => P::Done,
        _ => {
            return Err(TransitionError {
                phase,
                direction,
                frame,
            });
        }
    };
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admitted_session_and_repeated_output_artifacts() {
        use Direction::BuilderToClient as B;
        use Direction::ClientToBuilder as C;
        use FrameKind as F;
        let frames = [
            (C, F::Hello),
            (C, F::TraceContext),
            (C, F::AuthTicket),
            (B, F::AuthOk),
            (B, F::TraceContextAck),
            (C, F::BuildRequest),
            (C, F::InputManifest),
            (B, F::MissingInputs),
            (C, F::InputUpload),
            (B, F::BuildQueued),
            (B, F::BuildStarted),
            (B, F::BuildFinished),
            (B, F::OutputTransferArtifact),
            (B, F::OutputTransferArtifact),
            (B, F::OutputTransferDone),
            (B, F::Done),
        ];
        let mut phase = Phase::Open;
        for (direction, frame) in frames {
            phase = transition(phase, direction, frame).unwrap();
        }
        assert_eq!(phase, Phase::Done);
        assert_eq!(transition(phase, C, F::Error), Ok(Phase::Failed));
    }

    #[test]
    fn wrong_direction_order_and_unadmitted_data_frames_fail_closed() {
        let rejected = [
            (Phase::Open, Direction::BuilderToClient, FrameKind::Hello),
            (Phase::Open, Direction::ClientToBuilder, FrameKind::BuildRequest),
            (Phase::Building, Direction::ClientToBuilder, FrameKind::BuildFinished),
            (Phase::AwaitOutputTransfer, Direction::BuilderToClient, FrameKind::TransferManifest),
            (Phase::Failed, Direction::BuilderToClient, FrameKind::Done),
        ];
        for (phase, direction, frame) in rejected {
            let error = transition(phase, direction, frame).unwrap_err();
            assert_eq!(error.phase, phase);
            assert_eq!(error.direction, direction);
            assert_eq!(error.frame, frame);
            assert_eq!(
                error.diagnostic(),
                format!(
                    "unexpected-remote-frame phase={} direction={} frame={}",
                    phase.as_str(),
                    direction.as_str(),
                    frame.as_str()
                )
            );
        }
    }
}
