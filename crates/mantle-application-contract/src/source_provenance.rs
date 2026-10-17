//! SourceProvenance command family.
//!
//! SourceProvenance commands share one typed request, blocker set, port, and result, so
//! the composition root maps CLI DTOs into this command and renders the typed
//! result while the port owns the family's effects.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use mantle_rust_plan_core::Blake3Digest;

use crate::envelope::ApplicationBlocker;
use crate::envelope::CapabilityError;
use crate::envelope::EffectKind;
use crate::envelope::EffectMeasure;
use crate::envelope::EffectPlan;
use crate::envelope::EffectSpec;
use crate::envelope::ExpectedOutput;
use crate::envelope::plan_effects;
use crate::family::CommandFamily;

/// The source-bundle CLI operations that consume source, network, or state authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceBundleEffectAction {
    Plan,
    Export { fetch_missing: bool },
    List,
    Import { pin: bool },
    Verify { imported: bool },
    Preflight,
    BootstrapProfile { publish: bool, preflight: bool },
    Refresh,
    Hydrate,
}

/// An admitted source operation, before the first adapter effect runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceBundleEffectRequest {
    pub action: SourceBundleEffectAction,
    pub declared_input_count: u32,
}

/// The bounded number of records a source operation can consume or publish.
pub const MAX_SOURCE_BUNDLE_EFFECT_RECORDS: u32 = 65_536;
/// Maximum serialized source artifact bytes, allowing expansion of the 1 TiB
/// admitted payload into text encoding plus bounded record metadata.
pub const MAX_SOURCE_BUNDLE_EFFECT_BYTES: u64 = 4_398_046_511_104;

const READ_SOURCE_EFFECT: EffectSpec<'static> = EffectSpec {
    effect_id: "source-input",
    kind: EffectKind::ReadFiles,
    limit: EffectMeasure::Bytes(MAX_SOURCE_BUNDLE_EFFECT_BYTES),
    expected_output: ExpectedOutput::Identity("mantle-source-bundle-v1"),
};
const PREFLIGHT_SOURCE_EFFECT: EffectSpec<'static> = EffectSpec {
    effect_id: "source-input",
    kind: EffectKind::ReadFiles,
    limit: EffectMeasure::Bytes(MAX_SOURCE_BUNDLE_EFFECT_BYTES),
    expected_output: ExpectedOutput::Identity("mantle-source-offline-preflight-v1"),
};
const READ_STATE_EFFECT: EffectSpec<'static> = EffectSpec {
    effect_id: "source-state",
    kind: EffectKind::ReadFiles,
    limit: EffectMeasure::Items(MAX_SOURCE_BUNDLE_EFFECT_RECORDS),
    expected_output: ExpectedOutput::Identity("source-state"),
};
const WRITE_BUNDLE_EFFECT: EffectSpec<'static> = EffectSpec {
    effect_id: "source-publication",
    kind: EffectKind::WriteFiles,
    limit: EffectMeasure::Bytes(MAX_SOURCE_BUNDLE_EFFECT_BYTES),
    expected_output: ExpectedOutput::Identity("source-bundle"),
};
const WRITE_STATE_EFFECT: EffectSpec<'static> = EffectSpec {
    effect_id: "source-publication",
    kind: EffectKind::WriteFiles,
    limit: EffectMeasure::Items(MAX_SOURCE_BUNDLE_EFFECT_RECORDS),
    expected_output: ExpectedOutput::Identity("source-state"),
};
const CONFIRM_BUNDLE_EFFECT: EffectSpec<'static> = EffectSpec {
    effect_id: "source-readback",
    kind: EffectKind::ReadFiles,
    limit: EffectMeasure::Bytes(MAX_SOURCE_BUNDLE_EFFECT_BYTES),
    expected_output: ExpectedOutput::Identity("source-bundle"),
};
const FETCH_SOURCE_EFFECT: EffectSpec<'static> = EffectSpec {
    effect_id: "source-connected-fetch",
    kind: EffectKind::UseNetwork,
    limit: EffectMeasure::Calls(MAX_SOURCE_BUNDLE_EFFECT_RECORDS),
    expected_output: ExpectedOutput::None,
};
const HYDRATE_VENDOR_EFFECT: EffectSpec<'static> = EffectSpec {
    effect_id: "source-vendor-hydration",
    kind: EffectKind::WriteFiles,
    limit: EffectMeasure::Bytes(MAX_SOURCE_BUNDLE_EFFECT_BYTES),
    expected_output: ExpectedOutput::Identity("vendor-deps"),
};

// r[impl application_architecture.effect_observation_boundary]
/// Build a complete authority plan from caller declarations, before any I/O.
///
/// The identities describe artifacts, not an assertion that an effect ran. The
/// adapter must report its own kind, observed artifact, and measured usage.
pub fn plan_source_bundle_effects(request: SourceBundleEffectRequest) -> Result<EffectPlan, SourceProvenanceBlocker> {
    if request.declared_input_count > MAX_SOURCE_BUNDLE_EFFECT_RECORDS {
        return Err(SourceProvenanceBlocker::TooManyDeclaredEntries);
    }
    let read = READ_SOURCE_EFFECT;
    let preflight_read = PREFLIGHT_SOURCE_EFFECT;
    let state = READ_STATE_EFFECT;
    let write = WRITE_BUNDLE_EFFECT;
    let import_write = WRITE_STATE_EFFECT;
    let confirm = CONFIRM_BUNDLE_EFFECT;
    let network = FETCH_SOURCE_EFFECT;
    let hydrate = HYDRATE_VENDOR_EFFECT;
    let effects: &[EffectSpec<'_>] = match request.action {
        SourceBundleEffectAction::Plan | SourceBundleEffectAction::List => &[read],
        SourceBundleEffectAction::Export { fetch_missing: false } => &[read, write, confirm],
        SourceBundleEffectAction::Export { fetch_missing: true } => &[read, network, write, confirm],
        SourceBundleEffectAction::Import { .. } => &[read, import_write, state],
        SourceBundleEffectAction::Verify { imported: false } => &[read],
        SourceBundleEffectAction::Verify { imported: true }
        | SourceBundleEffectAction::BootstrapProfile {
            publish: false,
            preflight: true,
        } => &[read, state],
        SourceBundleEffectAction::Preflight => &[preflight_read, state],
        SourceBundleEffectAction::BootstrapProfile {
            publish: true,
            preflight: true,
        } => &[read, write, state, confirm],
        SourceBundleEffectAction::BootstrapProfile {
            publish: true,
            preflight: false,
        }
        | SourceBundleEffectAction::Refresh => &[read, write, confirm],
        SourceBundleEffectAction::BootstrapProfile {
            publish: false,
            preflight: false,
        } => &[read],
        SourceBundleEffectAction::Hydrate => &[read, hydrate, state],
    };
    plan_effects(CommandFamily::SourceProvenance, effects).map_err(|error| {
        SourceProvenanceBlocker::Domain(ApplicationBlocker::new(
            error.code(),
            "source_provenance",
            "source operation has an invalid or unbounded effect plan",
        ))
    })
}

/// Maximum admitted declared entries for one source provenance command.
pub const MAX_SOURCE_PROVENANCE_DECLARED_ENTRIES: u32 = 256;

/// Admitted blocker slots for one source provenance command beyond its declared entries.
const MAX_SOURCE_PROVENANCE_BLOCKERS: usize = 4;

/// SourceProvenance operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SourceProvenanceOperation {
    /// Admit operation.
    Admit,
    /// Bundle operation.
    Bundle,
    /// Hydrate operation.
    Hydrate,
    /// Attest operation.
    Attest,
    /// Export operation.
    Export,
}

impl SourceProvenanceOperation {
    /// Every operation in canonical order.
    pub fn all() -> Vec<Self> {
        let entries = vec![Self::Admit, Self::Bundle, Self::Hydrate, Self::Attest, Self::Export];
        debug_assert_eq!(entries.len(), 5);
        debug_assert!(!entries.is_empty());
        entries
    }

    /// Stable operation label.
    pub fn as_str(self) -> &'static str {
        let label = match self {
            Self::Admit => "admit",
            Self::Bundle => "bundle",
            Self::Hydrate => "hydrate",
            Self::Attest => "attest",
            Self::Export => "export",
        };
        debug_assert!(!label.is_empty());
        debug_assert!(SourceProvenanceOperation::all().contains(&self));
        label
    }

    /// Whether the operation requires at least one declared entry.
    pub fn requires_declared_entries(self) -> bool {
        let is_required = matches!(self, Self::Admit | Self::Hydrate | Self::Attest);
        debug_assert!(SourceProvenanceOperation::all().contains(&self));
        debug_assert!(!self.as_str().is_empty());
        is_required
    }
}

/// One typed SourceProvenance command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProvenanceCommand {
    /// Command root that produced this command.
    pub root: String,
    pub operation: SourceProvenanceOperation,
    /// Primary subject of the command, such as a path, name, or selector.
    pub subject: String,
    /// Declared entries the operation consumes, in caller order.
    pub declared_entries: Vec<String>,
    /// Expected digest.
    pub expected_digest: Option<Blake3Digest>,
}

/// Domain blocker specific to SourceProvenance commands.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SourceProvenanceBlocker {
    /// The request named no subject.
    MissingSubject,
    /// The operation needs a declared entry and the request named none.
    MissingDeclaredEntry,
    /// The request exceeded the admitted declared-entry bound.
    TooManyDeclaredEntries,
    /// The operation needs an input specific to this family.
    DigestRequired,
    /// Domain policy rejected the request for another reason.
    Domain(ApplicationBlocker),
}

/// Typed SourceProvenance result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProvenanceResult {
    /// Admitted records count.
    pub admitted_records_count: u32,
    /// Materialized bytes count.
    pub materialized_bytes_count: u64,
    /// Bundle identity.
    pub bundle_identity: Option<Blake3Digest>,
}

/// Terminal SourceProvenance outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceProvenanceOutcome {
    /// The operation ran and produced a result.
    Completed(SourceProvenanceResult),
    /// Domain policy rejected the request before any effect ran.
    Blocked(Vec<SourceProvenanceBlocker>),
}

/// Port: execute one typed SourceProvenance command.
pub trait SourceProvenancePort {
    fn run(&mut self, request: &SourceProvenanceCommand) -> Result<SourceProvenanceOutcome, CapabilityError>;
}

/// Validate one SourceProvenance command before any port is called.
pub fn validate_source_provenance(request: &SourceProvenanceCommand) -> Vec<SourceProvenanceBlocker> {
    let declared_count = request.declared_entries.len();
    let blocker_slots = declared_count.saturating_add(MAX_SOURCE_PROVENANCE_BLOCKERS);
    let mut blockers: Vec<SourceProvenanceBlocker> = Vec::with_capacity(blocker_slots);
    if request.root.is_empty() {
        blockers.push(SourceProvenanceBlocker::Domain(ApplicationBlocker::new(
            "missing-command-root",
            "source_provenance",
            "a command must name its command root",
        )));
    }
    if request.subject.trim().is_empty() {
        blockers.push(SourceProvenanceBlocker::MissingSubject);
    }
    if request.operation.requires_declared_entries() && declared_count == 0 {
        blockers.push(SourceProvenanceBlocker::MissingDeclaredEntry);
    }
    let is_declared_count_admissible =
        u32::try_from(declared_count).is_ok_and(|count| count <= MAX_SOURCE_PROVENANCE_DECLARED_ENTRIES);
    if !is_declared_count_admissible {
        blockers.push(SourceProvenanceBlocker::TooManyDeclaredEntries);
    }
    if matches!(request.operation, SourceProvenanceOperation::Attest | SourceProvenanceOperation::Hydrate)
        && request.expected_digest.is_none()
    {
        blockers.push(SourceProvenanceBlocker::DigestRequired);
    }
    blockers.sort();
    blockers.dedup();
    debug_assert!(blockers.len() <= declared_count.saturating_add(MAX_SOURCE_PROVENANCE_BLOCKERS));
    let empty_entries = request.declared_entries.iter().filter(|entry| entry.is_empty()).count();
    debug_assert!(empty_entries <= declared_count);
    blockers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> SourceProvenanceCommand {
        SourceProvenanceCommand {
            root: String::from("source-provenance"),
            operation: SourceProvenanceOperation::all()[0],
            subject: String::from("subject"),
            declared_entries: vec![String::from("first")],
            expected_digest: Some(Blake3Digest::from_slice(&[0u8; 32])),
        }
    }

    #[test]
    fn a_complete_request_is_admissible() {
        let request = sample_request();
        assert!(validate_source_provenance(&request).is_empty());
        assert_eq!(SourceProvenanceOperation::all().len(), 5);
        let declared_required = SourceProvenanceOperation::all()
            .iter()
            .filter(|operation| operation.requires_declared_entries())
            .count();
        assert_eq!(declared_required, 3);
    }

    #[test]
    fn missing_root_subject_and_declared_entries_are_rejected() {
        let mut request = sample_request();
        request.root = String::new();
        request.subject = String::from("   ");
        request.declared_entries.clear();
        let blockers = validate_source_provenance(&request);
        assert!(blockers.iter().any(|blocker| matches!(blocker, SourceProvenanceBlocker::Domain(_))));
        assert!(blockers.contains(&SourceProvenanceBlocker::MissingSubject));
        assert!(!SourceProvenanceOperation::all().is_empty());
    }

    #[test]
    fn the_family_specific_requirement_is_enforced() {
        let mut request = sample_request();
        request.expected_digest = None;
        request.operation = SourceProvenanceOperation::Attest;
        let blockers = validate_source_provenance(&request);
        assert!(blockers.contains(&SourceProvenanceBlocker::DigestRequired));
        assert!(!blockers.is_empty());
        let empty_labels = SourceProvenanceOperation::all()
            .iter()
            .map(|operation| operation.as_str())
            .filter(|label| label.is_empty())
            .count();
        assert_eq!(empty_labels, 0);
    }
    fn observed(effect_id: &str, kind: EffectKind, usage: EffectMeasure, output: &str) -> crate::envelope::Observation {
        crate::envelope::Observation {
            effect_id: crate::envelope::EffectId(String::from(effect_id)),
            kind,
            status: crate::envelope::ObservationStatus::Succeeded,
            output: crate::envelope::EffectOutput::Identity(String::from(output)),
            usage,
            diagnostics_code: None,
        }
    }

    #[test]
    fn export_admits_network_only_when_requested_and_all_observations_are_required() {
        let offline = plan_source_bundle_effects(SourceBundleEffectRequest {
            action: SourceBundleEffectAction::Export { fetch_missing: false },
            declared_input_count: 1,
        })
        .unwrap();
        let connected = plan_source_bundle_effects(SourceBundleEffectRequest {
            action: SourceBundleEffectAction::Export { fetch_missing: true },
            declared_input_count: 1,
        })
        .unwrap();
        assert_eq!(offline.effects.len(), 3);
        assert_eq!(connected.effects.len(), 4);
        assert!(!offline.effects.iter().any(|effect| effect.kind == EffectKind::UseNetwork));
        assert!(connected.effects.iter().any(|effect| effect.kind == EffectKind::UseNetwork));
        let observed_read =
            observed("source-input", EffectKind::ReadFiles, EffectMeasure::Bytes(1), "mantle-source-bundle-v1");
        let observed_write =
            observed("source-publication", EffectKind::WriteFiles, EffectMeasure::Bytes(1), "source-bundle");
        assert!(matches!(
            crate::envelope::classify_observations(&offline, &[observed_read, observed_write]),
            crate::envelope::ApplicationOutcome::Rejected {
                missing_effect_count: 1,
                ..
            }
        ));
        let connected_observations = [
            observed("source-input", EffectKind::ReadFiles, EffectMeasure::Bytes(1), "mantle-source-bundle-v1"),
            crate::envelope::Observation {
                effect_id: crate::envelope::EffectId(String::from("source-connected-fetch")),
                kind: EffectKind::UseNetwork,
                status: crate::envelope::ObservationStatus::Succeeded,
                output: crate::envelope::EffectOutput::None,
                usage: EffectMeasure::Calls(0),
                diagnostics_code: None,
            },
            observed("source-publication", EffectKind::WriteFiles, EffectMeasure::Bytes(1), "source-bundle"),
            observed("source-readback", EffectKind::ReadFiles, EffectMeasure::Bytes(1), "source-bundle"),
        ];
        assert!(matches!(
            crate::envelope::classify_observations(&connected, &connected_observations),
            crate::envelope::ApplicationOutcome::Completed
        ));
    }

    #[test]
    fn source_publication_rejects_wrong_artifact_and_excess_usage() {
        let plan = plan_source_bundle_effects(SourceBundleEffectRequest {
            action: SourceBundleEffectAction::Export { fetch_missing: false },
            declared_input_count: 1,
        })
        .unwrap();
        let observations = [
            observed("source-input", EffectKind::ReadFiles, EffectMeasure::Bytes(1), "mantle-source-bundle-v1"),
            observed("source-publication", EffectKind::WriteFiles, EffectMeasure::Bytes(1), "source-bundle"),
            observed("source-readback", EffectKind::ReadFiles, EffectMeasure::Bytes(1), "source-bundle"),
        ];
        assert!(matches!(
            crate::envelope::classify_observations(&plan, &observations),
            crate::envelope::ApplicationOutcome::Completed
        ));
        let mut wrong_artifact = observations.clone();
        wrong_artifact[2].output = crate::envelope::EffectOutput::Identity(String::from("incomplete-source-bundle"));
        assert!(matches!(
            crate::envelope::classify_observations(&plan, &wrong_artifact),
            crate::envelope::ApplicationOutcome::Contradicted { .. }
        ));
        let mut wrong_authority = observations.clone();
        wrong_authority[2].kind = EffectKind::WriteFiles;
        assert!(matches!(
            crate::envelope::classify_observations(&plan, &wrong_authority),
            crate::envelope::ApplicationOutcome::Contradicted { .. }
        ));
        let mut over_limit = observations;
        over_limit[1].usage = EffectMeasure::Bytes(MAX_SOURCE_BUNDLE_EFFECT_BYTES + 1);
        assert!(matches!(
            crate::envelope::classify_observations(&plan, &over_limit),
            crate::envelope::ApplicationOutcome::Contradicted { .. }
        ));
    }

    #[test]
    fn source_effect_planning_rejects_input_counts_before_admission() {
        assert!(matches!(
            plan_source_bundle_effects(SourceBundleEffectRequest {
                action: SourceBundleEffectAction::Hydrate,
                declared_input_count: MAX_SOURCE_BUNDLE_EFFECT_RECORDS + 1,
            }),
            Err(SourceProvenanceBlocker::TooManyDeclaredEntries)
        ));
        assert!(
            plan_source_bundle_effects(SourceBundleEffectRequest {
                action: SourceBundleEffectAction::Hydrate,
                declared_input_count: MAX_SOURCE_BUNDLE_EFFECT_RECORDS,
            })
            .is_ok()
        );
    }
}
