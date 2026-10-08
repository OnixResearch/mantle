//! Realization command family: the first fully specified application contract.
//!
//! The build, check, run, develop, shell, and filegen roots share one typed
//! request, blocker set, port, and result. The root maps CLI DTOs into this
//! command and renders the typed result; the port executes the realization.

use alloc::string::String;
use alloc::vec::Vec;

use mantle_rust_plan_core::Blake3Digest;
use mantle_rust_plan_core::BuildProfile;

use crate::envelope::ApplicationBlocker;
use crate::envelope::CapabilityError;

/// Maximum admitted realization roots.
pub const MAX_REALIZATION_ROOTS: u32 = 256;

/// Distinct blockers one realization command can report besides its roots.
const MAX_REALIZATION_BLOCKERS: usize = 4;

/// One typed realization command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealizeCommand {
    /// Command root that produced this command (`build`, `check`, ...).
    pub root: String,
    /// Requested root names in caller order.
    pub roots: Vec<String>,
    /// Build profile the operator declared, when one was declared.
    ///
    /// The CLI exposes no profile flag on the realization roots, so a command
    /// that maps a real invocation carries `None`; the field stays because a
    /// caller that does know its profile can declare it.
    pub profile: Option<BuildProfile>,
    /// Operator-requested job limit, when supplied.
    pub requested_jobs: Option<u32>,
    /// Whether the operator requested a dry run.
    pub dry_run: bool,
}

/// Domain blocker specific to realization.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RealizationBlocker {
    /// The request named no roots.
    MissingRoots,
    /// The request exceeded the admitted root bound.
    TooManyRoots,
    /// One requested root was blank after trimming.
    EmptyRoot,
    /// Domain policy rejected the request for another reason.
    Domain(ApplicationBlocker),
}

/// Typed realization result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealizeResult {
    /// Planned unit count.
    pub planned_units: u32,
    /// Executed unit count.
    pub executed_units: u32,
    /// Cache-hit unit count.
    pub cache_hits: u32,
    /// Output identities in canonical order.
    pub output_identities: Vec<Blake3Digest>,
}

/// Terminal realization outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RealizeOutcome {
    /// The plan executed and produced a result.
    Completed(RealizeResult),
    /// Domain policy rejected the request before any effect ran.
    Blocked(Vec<RealizationBlocker>),
}

/// Port: execute one typed realization command.
pub trait RealizePort {
    fn realize(&mut self, command: &RealizeCommand) -> Result<RealizeOutcome, CapabilityError>;
}

/// Validate one realization command before any port is called.
pub fn validate_realize_command(command: &RealizeCommand) -> Vec<RealizationBlocker> {
    let root_count = command.roots.len();
    let blocker_slots = root_count.saturating_add(MAX_REALIZATION_BLOCKERS);
    let mut blockers: Vec<RealizationBlocker> = Vec::with_capacity(blocker_slots);
    if command.root.is_empty() {
        blockers.push(RealizationBlocker::Domain(ApplicationBlocker::new(
            "missing-command-root",
            "realize",
            "a realization command must name its command root",
        )));
    }
    if command.roots.is_empty() {
        blockers.push(RealizationBlocker::MissingRoots);
    }
    let is_root_count_admissible = u32::try_from(root_count).is_ok_and(|count| count <= MAX_REALIZATION_ROOTS);
    if !is_root_count_admissible {
        blockers.push(RealizationBlocker::TooManyRoots);
    }
    for root in &command.roots {
        if root.trim().is_empty() {
            blockers.push(RealizationBlocker::EmptyRoot);
        }
    }
    blockers.sort();
    blockers.dedup();
    debug_assert!(blockers.len() <= blocker_slots);
    let empty_roots = command.roots.iter().filter(|root| root.trim().is_empty()).count();
    debug_assert!(empty_roots <= root_count);
    blockers
}
