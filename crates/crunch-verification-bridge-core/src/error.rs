//! Failures while linking externally authored producer evidence by identity.
use alloc::string::String;
use core::fmt;

use serde_json::Error;

#[derive(Debug)]
pub enum BridgeError {
    DocumentByteBound,
    InvalidSourceRevision,
    InvalidPackagingIdentity,
    MaterialCountBound,
    MaterialSizeOverflow,
    TotalMaterialByteBound,
    UnboundExternalReceipt,
    MissingVerificationReport,
    UnsupportedVerificationSchema,
    VerificationDiagnostics,
    MissingVerifiedArtifacts,
    VerifiedArtifactCountBound,
    MissingEnvelopeArtifacts,
    MissingConsumerBinding,
    MissingSourcePaths,
    MissingProfileSource,
    MissingResourcePolicy,
    MissingStageArtifacts,
    StageArtifactCountBound,
    MissingArtifactIdentities,
    ArtifactIdentityCountBound,
    MissingProviderTools,
    MissingExtractionOptions,
    MissingTrellisArtifacts,
    MissingProfileArtifacts,
    ProfileArtifactMismatch,
    ProducerJobRequiresVerification,
    ProducerNetworkNotDenied,
    ToolGraphByteBound,
    InvalidToolGraphShape,
    ToolGraphRootOrCountBound,
    InvalidProducerDerivation,
    MissingToolInputDerivations,
    IncompleteToolClosure,
    MissingToolInputSources,
    NonStoreToolSource,
    UnsafeArtifactPath(String),
    UnsafeExternalArtifactPath(String),
    MissingExternalField(String),
    /// The complete mismatch diagnostic, retained in one allocation rather than
    /// separately copying the borrowed field name and both compared values.
    FieldMismatch(String),
    InvalidMaterial(String),
    DuplicateMaterial(String),
    MissingPayload(String),
    MissingStageEnvelope(String),
    ArtifactByteCountMismatch(String),
    ExternalJson(Error),
    ToolGraphJson(Error),
    Serialization(Error),
}

impl fmt::Display for BridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DocumentByteBound => formatter.write_str("document exceeds bridge byte bound or is empty"),
            Self::InvalidSourceRevision => formatter.write_str("source_revision must be an immutable Git object ID"),
            Self::InvalidPackagingIdentity => formatter.write_str("packaging identities must be canonical b3 digests"),
            Self::MaterialCountBound => formatter.write_str("material count exceeds bridge bound or is empty"),
            Self::MaterialSizeOverflow => formatter.write_str("material size overflow"),
            Self::TotalMaterialByteBound => formatter.write_str("aggregate material byte bound exceeded"),
            Self::UnboundExternalReceipt => {
                formatter.write_str("Octet report does not bind the exact external receipt")
            }
            Self::MissingVerificationReport => formatter.write_str("missing external artifact-verification report"),
            Self::UnsupportedVerificationSchema => {
                formatter.write_str("unsupported Octet artifact-verification schema")
            }
            Self::VerificationDiagnostics => formatter.write_str("external artifact verification has diagnostics"),
            Self::MissingVerifiedArtifacts => formatter.write_str("missing verified artifacts"),
            Self::VerifiedArtifactCountBound => formatter.write_str("invalid verified artifact count"),
            Self::MissingEnvelopeArtifacts => formatter.write_str("missing envelope artifacts"),
            Self::MissingConsumerBinding => formatter.write_str("missing Octet consumer binding"),
            Self::MissingSourcePaths => formatter.write_str("missing source paths"),
            Self::MissingProfileSource => formatter.write_str("source absent from producer profile"),
            Self::MissingResourcePolicy => formatter.write_str("missing resource policy"),
            Self::MissingStageArtifacts => formatter.write_str("missing stage artifacts"),
            Self::StageArtifactCountBound => formatter.write_str("invalid stage artifact count"),
            Self::MissingArtifactIdentities => formatter.write_str("missing artifact identities"),
            Self::ArtifactIdentityCountBound => formatter.write_str("artifact identity count exceeds bound"),
            Self::MissingProviderTools => formatter.write_str("missing provider tools"),
            Self::MissingExtractionOptions => formatter.write_str("missing extraction options"),
            Self::MissingTrellisArtifacts => formatter.write_str("missing Trellis artifacts"),
            Self::MissingProfileArtifacts => formatter.write_str("missing profile artifacts"),
            Self::ProfileArtifactMismatch => formatter.write_str("receipt artifacts differ from producer profile"),
            Self::ProducerJobRequiresVerification => {
                formatter.write_str("producer job binding requires the exact external verification report")
            }
            Self::ProducerNetworkNotDenied => {
                formatter.write_str("selected producer job does not declare denied-network execution")
            }
            Self::ToolGraphByteBound => formatter.write_str("tool graph byte bound exceeded or empty"),
            Self::InvalidToolGraphShape => formatter.write_str("tool graph must be Nix derivation-show JSON"),
            Self::ToolGraphRootOrCountBound => {
                formatter.write_str("tool graph root absent or derivation bound exceeded")
            }
            Self::InvalidProducerDerivation => formatter.write_str("invalid producer derivation identity"),
            Self::MissingToolInputDerivations => formatter.write_str("missing tool inputDrvs"),
            Self::IncompleteToolClosure => formatter.write_str("producer tool dependency closure is incomplete"),
            Self::MissingToolInputSources => formatter.write_str("missing tool inputSrcs"),
            Self::NonStoreToolSource => formatter.write_str("producer graph contains a non-store source input"),
            Self::UnsafeArtifactPath(path) => write!(formatter, "unsafe artifact path {path}"),
            Self::UnsafeExternalArtifactPath(path) => write!(formatter, "unsafe external artifact path {path}"),
            Self::MissingExternalField(pointer) => write!(formatter, "missing external field {pointer}"),
            Self::FieldMismatch(diagnostic) => formatter.write_str(diagnostic),
            Self::InvalidMaterial(path) => write!(formatter, "invalid or oversized material {path}"),
            Self::DuplicateMaterial(path) => write!(formatter, "duplicate material {path}"),
            Self::MissingPayload(path) => write!(formatter, "missing payload {path}"),
            Self::MissingStageEnvelope(role) => write!(formatter, "stage {role} has no envelope artifact"),
            Self::ArtifactByteCountMismatch(path) => write!(formatter, "artifact byte count mismatch: {path}"),
            Self::ExternalJson(error) => write!(formatter, "external JSON: {error}"),
            Self::ToolGraphJson(error) => write!(formatter, "tool graph JSON: {error}"),
            Self::Serialization(error) => fmt::Display::fmt(error, formatter),
        }
    }
}

impl core::error::Error for BridgeError {}
