//! Typed artifact operations: effect plans, capability-scoped ports, and
//! observation classification.
//!
//! `artifact import`, `artifact oci-export`, and `artifact oci-import` each run
//! as one composite port call around existing adapters:
//!
//! - import runs the store import, which reads the declared source tree and writes the artifact
//!   store, and then writes the requested import report;
//! - oci-export runs the layout export, which reads the declared projection, spec material, source
//!   admissions, and stored objects, and writes a new layout directory;
//! - oci-import runs the layout import, which reads and validates the declared layout and writes
//!   the store and the import report.
//!
//! The plan names each composite port call with its actual filesystem/store authority and
//! declared destination before it runs. Executed adapters supply independent observations
//! for classification; a port result alone does not attest where or how it ran.
//!
//! `artifact oci-push` and `oci-pull` plan three effects, one per port call:
//! the input read (the receipt probe and the trust policy, plus the signing
//! keys for a push), the registry transfer, and the receipt write. Target
//! admission, which is pure, runs between the read and the transfer. The pull
//! transfer is composite as well. The pull adapter fetches the image, then
//! materializes and admits the fetched layout and writes its import report,
//! all in one call. An effect after the point where a transfer stopped is
//! recorded as skipped.
//!
//! `artifact export` plans its effects under their own identities, because
//! the store copy and the receipt are both writes, separated by the export
//! decision. The plan has two effects, plus a receipt write when requested:
//!
//! - the attestation read;
//! - the content resolution, which reads when it probes a declared materialized path and writes
//!   when it copies the stored artifact into the declared output;
//! - the receipt write, planned only when a receipt is requested.
//!
//! When the export decision rejects the resolved content, the content effect
//! is observed as failed.
//!
//! Non-claims: an observation covers a whole port call. It does not say
//! whether a failure happened while reading, validating, transferring, or
//! writing, and an adapter's own admission check is observed as a failed
//! call, not as a blocked outcome. A failed observation also does not mean
//! nothing was written. For example, the import report write runs after the
//! store import, and a store copy stays in place when its content is then
//! found inadmissible.

#[cfg(test)]
use alloc::string::String;

#[cfg(test)]
use crate::envelope::ApplicationOutcome;
use crate::envelope::CapabilityError;
#[cfg(test)]
use crate::envelope::EffectId;
use crate::envelope::EffectKind;
use crate::envelope::EffectMeasure;
use crate::envelope::EffectOutput;
use crate::envelope::EffectPlan;
use crate::envelope::EffectSpec;
use crate::envelope::ExpectedOutput;
#[cfg(test)]
use crate::envelope::Observation;
#[cfg(test)]
use crate::envelope::ObservationStatus;
#[cfg(test)]
use crate::envelope::classify_observations;
use crate::envelope::plan_effects;
use crate::family::CommandFamily;

/// Effect identity of one composite artifact call: it reads the declared
/// inputs and writes the store, layout, or report.
pub const ARTIFACT_CALL_EFFECT: &str = "artifact-call";

/// Effect identity of a registry transfer's input read.
pub const ARTIFACT_REGISTRY_INPUT_EFFECT: &str = "read-files";

/// Effect identity of the registry transfer.
pub const ARTIFACT_REGISTRY_TRANSFER_EFFECT: &str = "use-network";

/// Effect identity of a registry transfer's receipt write.
pub const ARTIFACT_REGISTRY_RECEIPT_EFFECT: &str = "write-files";

/// Effect identity of the export's attestation read.
pub const ARTIFACT_EXPORT_ATTESTATION_EFFECT: &str = "read-attestation";

/// Effect identity of the export's content resolution: a probe of the declared
/// materialized path, or a copy of the stored artifact into the declared
/// output.
pub const ARTIFACT_EXPORT_CONTENT_EFFECT: &str = "resolve-content";

/// Effect identity of the export receipt write.
pub const ARTIFACT_EXPORT_RECEIPT_EFFECT: &str = "write-receipt";

/// Facts observed by an artifact capability adapter during one executed port call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactPortFact {
    pub kind: EffectKind,
    pub output: EffectOutput,
    pub usage: EffectMeasure,
}

/// A port call keeps its independently observed effect facts even when it fails.
#[derive(Debug)]
pub struct ArtifactPortCall<T> {
    pub result: Result<T, CapabilityError>,
    pub fact: ArtifactPortFact,
}

/// Port: import the one declared source into the artifact store, then write
/// the requested import report.
pub trait ArtifactImportPort {
    /// Report of one import, owned by the shell.
    type Report;

    /// Run the store import, then the requested report write.
    fn import_artifact(&mut self) -> ArtifactPortCall<Self::Report>;
}

/// Port: export the one declared projection as a new OCI image layout.
pub trait OciLayoutExportPort {
    /// Report of one export, owned by the shell.
    type Report;

    /// Run the layout export.
    fn export_layout(&mut self) -> ArtifactPortCall<Self::Report>;
}

/// Port: import the one declared OCI image layout and write its import report.
pub trait OciLayoutImportPort {
    /// Report of one import, owned by the shell.
    type Report;

    /// Run the layout import, including its report write.
    fn import_layout(&mut self) -> ArtifactPortCall<Self::Report>;
}

/// Port: one `artifact oci-push`, covering its declared inputs, the registry
/// push, and the receipt.
pub trait OciRegistryPushPort {
    /// Loaded push inputs, owned by the shell.
    type Inputs;
    /// One admitted push: the loaded inputs and the validated target.
    type Admitted;
    /// Report of one push, owned by the shell.
    type Report;

    /// Confirm that the receipt path is free, then read the trust policy and
    /// the signing keys.
    fn read_inputs(&mut self) -> ArtifactPortCall<Self::Inputs>;

    /// Publish the layout.
    fn push(&mut self, admitted: &Self::Admitted) -> ArtifactPortCall<Self::Report>;

    /// Write the receipt of a finished push.
    fn write_receipt(&mut self, report: &Self::Report) -> ArtifactPortCall<()>;
}

/// Port: one `artifact oci-pull`, covering its declared inputs, the registry
/// pull, and the receipt.
pub trait OciRegistryPullPort {
    /// Loaded pull inputs, owned by the shell.
    type Inputs;
    /// One admitted pull: the loaded inputs and the validated target.
    type Admitted;
    /// Report of one pull, owned by the shell.
    type Report;

    /// Confirm that the receipt path is free, then read the trust policy.
    fn read_inputs(&mut self) -> ArtifactPortCall<Self::Inputs>;

    /// Fetch the image, then materialize and admit the fetched layout and
    /// write its import report.
    fn pull(&mut self, admitted: &Self::Admitted) -> ArtifactPortCall<Self::Report>;

    /// Write the receipt of a finished pull.
    fn write_receipt(&mut self, report: &Self::Report) -> ArtifactPortCall<()>;
}

/// Port: one `artifact export`, covering its declared attestation, its
/// content, and its receipt.
pub trait ArtifactExportPort {
    /// Loaded attestation, owned by the shell.
    type Attestation;
    /// Resolved content facts, owned by the shell; absent content is a value,
    /// not an error.
    type Content;
    /// Export receipt, owned by the shell.
    type Receipt;

    /// Read the declared attestation.
    fn read_attestation(&mut self) -> ArtifactPortCall<Self::Attestation>;

    /// Probe the declared materialized path, or copy the stored artifact into
    /// the declared output.
    fn resolve_content(&mut self) -> ArtifactPortCall<Self::Content>;

    /// Write the receipt of an admitted export.
    fn write_receipt(&mut self, receipt: &Self::Receipt) -> ArtifactPortCall<()>;
}

/// Where one export's content comes from, fixed by its request before any
/// effect runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactExportContentSource {
    /// Probe an already-materialized path, which only reads.
    MaterializedPath,
    /// Copy the stored artifact into the declared output, which writes.
    Store,
}

/// The bounded plan for one composite artifact call and its declared destination.
pub fn artifact_call_effect_plan(kind: EffectKind, destination: &str) -> Result<EffectPlan, CapabilityError> {
    plan_effects(CommandFamily::SourceProvenance, &[spec(ARTIFACT_CALL_EFFECT, kind, Some(destination))])
        .map_err(|error| CapabilityError::new(error.code(), "the artifact call plan was rejected"))
}

/// The bounded plan for one registry transfer: input, network, and receipt.
pub fn artifact_registry_effect_plan(receipt_destination: &str) -> Result<EffectPlan, CapabilityError> {
    plan_effects(CommandFamily::SourceProvenance, &[
        spec(ARTIFACT_REGISTRY_INPUT_EFFECT, EffectKind::ReadFiles, None),
        spec(ARTIFACT_REGISTRY_TRANSFER_EFFECT, EffectKind::UseNetwork, None),
        spec(ARTIFACT_REGISTRY_RECEIPT_EFFECT, EffectKind::WriteFiles, Some(receipt_destination)),
    ])
    .map_err(|error| CapabilityError::new(error.code(), "the artifact registry plan was rejected"))
}

/// The attestation read and content resolution precede any receipt write.
pub fn artifact_export_effect_plan(
    source: ArtifactExportContentSource,
    attestation: &str,
    content_destination: Option<&str>,
    receipt_destination: Option<&str>,
) -> Result<EffectPlan, CapabilityError> {
    let content_kind = match source {
        ArtifactExportContentSource::MaterializedPath => EffectKind::ReadFiles,
        ArtifactExportContentSource::Store => EffectKind::StoreAccess,
    };
    let declarations = [
        spec(ARTIFACT_EXPORT_ATTESTATION_EFFECT, EffectKind::ReadFiles, Some(attestation)),
        spec(ARTIFACT_EXPORT_CONTENT_EFFECT, content_kind, content_destination),
        spec(ARTIFACT_EXPORT_RECEIPT_EFFECT, EffectKind::WriteFiles, receipt_destination),
    ];
    let count = if receipt_destination.is_some() { 3 } else { 2 };
    plan_effects(CommandFamily::SourceProvenance, &declarations[..count])
        .map_err(|error| CapabilityError::new(error.code(), "the artifact export plan was rejected"))
}

fn spec<'a>(effect_id: &'a str, kind: EffectKind, output: Option<&'a str>) -> EffectSpec<'a> {
    EffectSpec {
        effect_id,
        kind,
        limit: EffectMeasure::Calls(1),
        expected_output: output.map_or(ExpectedOutput::None, ExpectedOutput::Identity),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observed(
        effect: &str,
        kind: EffectKind,
        output: EffectOutput,
        status: ObservationStatus,
        calls: u32,
    ) -> Observation {
        Observation {
            effect_id: EffectId(String::from(effect)),
            kind,
            status,
            output,
            usage: EffectMeasure::Calls(calls),
            diagnostics_code: None,
        }
    }

    #[test]
    fn composite_store_call_checks_independent_authority_destination_and_bound() {
        let plan = artifact_call_effect_plan(EffectKind::StoreAccess, "/state/artifacts").unwrap();
        let actual = observed(
            ARTIFACT_CALL_EFFECT,
            EffectKind::StoreAccess,
            EffectOutput::Identity(String::from("/state/artifacts")),
            ObservationStatus::Succeeded,
            1,
        );
        assert_eq!(classify_observations(&plan, core::slice::from_ref(&actual)), ApplicationOutcome::Completed);
        for wrong in [
            Observation {
                kind: EffectKind::ReadFiles,
                ..actual.clone()
            },
            Observation {
                output: EffectOutput::Identity(String::from("/other/artifacts")),
                ..actual.clone()
            },
            Observation {
                usage: EffectMeasure::Calls(2),
                ..actual.clone()
            },
        ] {
            assert_eq!(classify_observations(&plan, &[wrong]), ApplicationOutcome::Contradicted { effect_count: 1 });
        }
    }

    #[test]
    fn registry_stops_and_receipt_identity_cannot_be_invented() {
        let plan = artifact_registry_effect_plan("/receipts/push.json").unwrap();
        let read = observed(
            ARTIFACT_REGISTRY_INPUT_EFFECT,
            EffectKind::ReadFiles,
            EffectOutput::None,
            ObservationStatus::Succeeded,
            1,
        );
        let transfer = observed(
            ARTIFACT_REGISTRY_TRANSFER_EFFECT,
            EffectKind::UseNetwork,
            EffectOutput::None,
            ObservationStatus::Succeeded,
            1,
        );
        let receipt = observed(
            ARTIFACT_REGISTRY_RECEIPT_EFFECT,
            EffectKind::WriteFiles,
            EffectOutput::Identity(String::from("/receipts/push.json")),
            ObservationStatus::Succeeded,
            1,
        );
        assert_eq!(
            classify_observations(&plan, &[read.clone(), transfer.clone(), receipt.clone()]),
            ApplicationOutcome::Completed
        );
        let skipped = observed(
            ARTIFACT_REGISTRY_RECEIPT_EFFECT,
            EffectKind::WriteFiles,
            EffectOutput::None,
            ObservationStatus::Skipped,
            0,
        );
        assert_eq!(
            classify_observations(&plan, &[read.clone(), transfer.clone(), skipped]),
            ApplicationOutcome::Failed { failed_effect_count: 1 }
        );
        for wrong in [
            Observation {
                kind: EffectKind::ReadFiles,
                ..receipt.clone()
            },
            Observation {
                output: EffectOutput::Identity(String::from("/receipts/elsewhere.json")),
                ..receipt.clone()
            },
            Observation {
                usage: EffectMeasure::Items(1),
                ..receipt.clone()
            },
        ] {
            assert_eq!(
                classify_observations(&plan, &[read.clone(), transfer.clone(), wrong]),
                ApplicationOutcome::Contradicted { effect_count: 1 }
            );
        }
    }

    #[test]
    fn export_store_copy_and_receipt_reject_wrong_observed_facts() {
        let plan = artifact_export_effect_plan(
            ArtifactExportContentSource::Store,
            "/input/attestation.json",
            Some("/output/artifact"),
            Some("/output/receipt.json"),
        )
        .unwrap();
        let attestation = observed(
            ARTIFACT_EXPORT_ATTESTATION_EFFECT,
            EffectKind::ReadFiles,
            EffectOutput::Identity(String::from("/input/attestation.json")),
            ObservationStatus::Succeeded,
            1,
        );
        let content = observed(
            ARTIFACT_EXPORT_CONTENT_EFFECT,
            EffectKind::StoreAccess,
            EffectOutput::Identity(String::from("/output/artifact")),
            ObservationStatus::Succeeded,
            1,
        );
        let receipt = observed(
            ARTIFACT_EXPORT_RECEIPT_EFFECT,
            EffectKind::WriteFiles,
            EffectOutput::Identity(String::from("/output/receipt.json")),
            ObservationStatus::Succeeded,
            1,
        );
        assert_eq!(
            classify_observations(&plan, &[attestation.clone(), content.clone(), receipt.clone()]),
            ApplicationOutcome::Completed
        );
        for wrong in [
            Observation {
                kind: EffectKind::ReadFiles,
                ..content.clone()
            },
            Observation {
                output: EffectOutput::Identity(String::from("/other/artifact")),
                ..content.clone()
            },
            Observation {
                usage: EffectMeasure::Calls(2),
                ..content.clone()
            },
        ] {
            assert_eq!(
                classify_observations(&plan, &[attestation.clone(), wrong, receipt.clone()]),
                ApplicationOutcome::Contradicted { effect_count: 1 }
            );
        }
        let stopped = observed(
            ARTIFACT_EXPORT_CONTENT_EFFECT,
            EffectKind::StoreAccess,
            EffectOutput::None,
            ObservationStatus::Failed,
            1,
        );
        let skipped = observed(
            ARTIFACT_EXPORT_RECEIPT_EFFECT,
            EffectKind::WriteFiles,
            EffectOutput::None,
            ObservationStatus::Skipped,
            0,
        );
        assert_eq!(classify_observations(&plan, &[attestation, stopped, skipped]), ApplicationOutcome::Failed {
            failed_effect_count: 2
        });
    }
}
