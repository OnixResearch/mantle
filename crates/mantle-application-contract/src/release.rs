//! Release command family.
//!
//! Release creation, verification, attestation, and witness flows share one
//! typed request, blocker set, port, and result. Proof kinds are a closed set
//! so an unknown proof cannot be counted as evidence.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use mantle_rust_plan_core::Blake3Digest;

use crate::envelope::ApplicationBlocker;
use crate::envelope::CapabilityError;

/// Maximum admitted required proofs for one release command.
pub const MAX_RELEASE_PROOFS: u32 = 64;

/// Admitted blocker slots for one release command.
const MAX_RELEASE_BLOCKERS: usize = 3;

/// Release operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReleaseOperation {
    /// Create a release-evidence bundle.
    Create,
    /// Verify a release-evidence bundle.
    Verify,
    /// Sign a release attestation.
    Attest,
    /// Rebuild a witness from a request directory.
    WitnessRebuild,
    /// Import returned witness sidecars.
    WitnessImport,
    /// Export a public verification request.
    WitnessExport,
}

impl ReleaseOperation {
    /// Every release operation in canonical order.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Create,
            Self::Verify,
            Self::Attest,
            Self::WitnessRebuild,
            Self::WitnessImport,
            Self::WitnessExport,
        ]
    }

    /// Stable operation label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Verify => "verify",
            Self::Attest => "attest",
            Self::WitnessRebuild => "witness-rebuild",
            Self::WitnessImport => "witness-import",
            Self::WitnessExport => "witness-export",
        }
    }

    /// Whether the operation consumes a bundle directory.
    pub fn requires_bundle(self) -> bool {
        let is_bundle_required = !matches!(self, Self::Create);
        debug_assert!(is_bundle_required || Self::all().contains(&self));
        debug_assert!(!self.as_str().is_empty());
        is_bundle_required
    }
}

/// Admitted release proof kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReleaseProofKind {
    SelfHostingProof,
    FixedPoint,
    BootstrapParity,
    WitnessRebuild,
}

impl ReleaseProofKind {
    /// Every admitted proof kind in canonical order.
    pub fn all() -> Vec<Self> {
        vec![
            Self::SelfHostingProof,
            Self::FixedPoint,
            Self::BootstrapParity,
            Self::WitnessRebuild,
        ]
    }

    /// Stable proof label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SelfHostingProof => "self-hosting-proof",
            Self::FixedPoint => "fixed-point",
            Self::BootstrapParity => "bootstrap-parity",
            Self::WitnessRebuild => "witness-rebuild",
        }
    }

    /// Parse one proof label, rejecting unknown kinds.
    pub fn parse(label: &str) -> Option<Self> {
        let parsed = Self::all().into_iter().find(|kind| kind.as_str() == label);
        debug_assert!(parsed.is_none() || !label.is_empty());
        debug_assert!(Self::all().iter().all(|kind| !kind.as_str().is_empty()));
        parsed
    }
}

/// One typed release command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseCommand {
    /// Command root that produced this command.
    pub root: String,
    pub operation: ReleaseOperation,
    /// Release-evidence bundle directory for bundle-consuming operations.
    pub bundle_dir: String,
    /// Required proof kinds in caller order.
    pub required_proofs: Vec<ReleaseProofKind>,
    /// Whether the operator requested a dry run.
    pub dry_run: bool,
}

/// Domain blocker specific to releases.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReleaseBlocker {
    /// The operation needs a bundle and the request named none.
    MissingBundle,
    /// The request exceeded the admitted proof bound.
    TooManyProofs,
    /// Domain policy rejected the request for another reason.
    Domain(ApplicationBlocker),
}

/// Typed release result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseResult {
    /// Bundle identity when the operation produced or verified one.
    pub bundle_identity: Option<Blake3Digest>,
    /// Verified proof count.
    pub verified_proofs: u32,
    /// Witness signatures counted.
    pub witness_count: u32,
    /// Witness signatures counted inside the independent domain.
    pub independent_witness_count: u32,
}

/// Terminal release outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseOutcome {
    /// The operation ran and produced a result.
    Completed(ReleaseResult),
    /// Domain policy rejected the request before any effect ran.
    Blocked(Vec<ReleaseBlocker>),
}

/// Port: execute one typed release command.
pub trait ReleasePort {
    fn release(&mut self, command: &ReleaseCommand) -> Result<ReleaseOutcome, CapabilityError>;
}

/// Validate one release command before any port is called.
pub fn validate_release_command(command: &ReleaseCommand) -> Vec<ReleaseBlocker> {
    let mut blockers: Vec<ReleaseBlocker> = Vec::with_capacity(MAX_RELEASE_BLOCKERS);
    if command.root.is_empty() {
        blockers.push(ReleaseBlocker::Domain(ApplicationBlocker::new(
            "missing-command-root",
            "release",
            "a release command must name its command root",
        )));
    }
    if command.bundle_dir.is_empty() && command.operation.requires_bundle() {
        blockers.push(ReleaseBlocker::MissingBundle);
    }
    let is_proof_count_admissible =
        u32::try_from(command.required_proofs.len()).is_ok_and(|count| count <= MAX_RELEASE_PROOFS);
    if !is_proof_count_admissible {
        blockers.push(ReleaseBlocker::TooManyProofs);
    }
    blockers.sort();
    blockers.dedup();
    debug_assert!(blockers.len() <= MAX_RELEASE_BLOCKERS);
    blockers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command(operation: ReleaseOperation, bundle_dir: &str, proofs: &[ReleaseProofKind]) -> ReleaseCommand {
        ReleaseCommand {
            root: String::from("release"),
            operation,
            bundle_dir: String::from(bundle_dir),
            required_proofs: proofs.to_vec(),
            dry_run: false,
        }
    }

    #[test]
    fn a_verify_command_with_a_bundle_and_known_proofs_is_admissible() {
        let request =
            command(ReleaseOperation::Verify, "target/release-evidence/run-1", &[ReleaseProofKind::WitnessRebuild]);
        assert!(validate_release_command(&request).is_empty());
        assert!(ReleaseOperation::Verify.requires_bundle());
    }

    #[test]
    fn a_bundle_consuming_operation_without_a_bundle_is_rejected() {
        let request = command(ReleaseOperation::Attest, "", &[]);
        let blockers = validate_release_command(&request);
        assert_eq!(blockers, vec![ReleaseBlocker::MissingBundle]);
        assert_eq!(ReleaseOperation::Attest.as_str(), "attest");
    }

    #[test]
    fn create_may_run_without_a_bundle_but_needs_its_root() {
        let mut request = command(ReleaseOperation::Create, "", &[ReleaseProofKind::FixedPoint]);
        assert!(validate_release_command(&request).is_empty());
        request.root = String::new();
        let blockers = validate_release_command(&request);
        assert!(blockers.iter().any(|blocker| matches!(blocker, ReleaseBlocker::Domain(_))));
        assert!(!ReleaseOperation::Create.requires_bundle());
    }

    #[test]
    fn proof_kinds_are_a_closed_set() {
        assert_eq!(ReleaseProofKind::parse("witness-rebuild"), Some(ReleaseProofKind::WitnessRebuild));
        assert_eq!(ReleaseProofKind::parse("self-hosting-proof"), Some(ReleaseProofKind::SelfHostingProof));
        assert_eq!(ReleaseProofKind::parse("made-up-proof"), None);
        assert_eq!(ReleaseProofKind::parse(""), None);
        assert_eq!(ReleaseProofKind::all().len(), 4);
        assert_eq!(ReleaseOperation::all().len(), 6);
    }
}
