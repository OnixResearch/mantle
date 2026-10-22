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

/// Capabilities used by build and interactive realization applications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealizationEffect {
    /// Composite local worker/store execution; completion requires materialized outputs.
    Build,
    /// Own a watched worker until requested shutdown and completed teardown.
    Watch,
    /// Open the selected backend and admit signing/trust configuration.
    Preparation,
    /// Resolve a project target against its manifest and import surface.
    ProjectRead,
    /// Read the bounded manifest/lock retention facts.
    RetentionRead,
    /// Resolve or materialize the embedded evaluator standard library.
    ImportPreparation,
    /// Select an executable from the materialized output tree.
    BinarySelection,
    /// Inspect the selected realized output before interactive execution.
    OutputSelection,
    /// Check inherited compile-driver runtime configuration before loading it.
    RuntimeEnvironment,
    /// Read and validate an explicit compile-driver runtime declaration.
    RuntimeRead,
    /// Persist build diagnostic logs; failures remain explicit advisory facts.
    Reporting,
    /// Publish and verify the requested build interchange records.
    Interchange,
    /// Read wall-clock time for a watch-session identity.
    Clock,
    /// Read and decode the shell activation sidecar.
    Activation,
    /// Snapshot the inherited activation environment.
    Environment,
    /// Run an executable and observe its actual termination status.
    Child,
}

/// The effect identity and authority of one realization capability.
fn realization_effect_authority(effect: RealizationEffect) -> (&'static str, crate::EffectKind) {
    let (effect_id, kind) = match effect {
        RealizationEffect::Watch => ("realization-watch", crate::EffectKind::StoreAccess),
        RealizationEffect::Build => ("realization-build", crate::EffectKind::StoreAccess),
        RealizationEffect::Preparation => ("realization-preparation", crate::EffectKind::StoreAccess),
        RealizationEffect::ProjectRead => ("realization-project", crate::EffectKind::ReadFiles),
        RealizationEffect::RetentionRead => ("realization-retention", crate::EffectKind::ReadFiles),
        RealizationEffect::OutputSelection => ("realization-output", crate::EffectKind::ReadFiles),
        RealizationEffect::ImportPreparation => ("realization-imports", crate::EffectKind::WriteFiles),
        RealizationEffect::BinarySelection => ("realization-binary", crate::EffectKind::ReadFiles),
        RealizationEffect::RuntimeEnvironment => {
            ("realization-runtime-environment", crate::EffectKind::ReadEnvironment)
        }
        RealizationEffect::RuntimeRead => ("realization-runtime-read", crate::EffectKind::ReadFiles),
        RealizationEffect::Reporting => ("realization-reporting", crate::EffectKind::WriteFiles),
        RealizationEffect::Interchange => ("realization-interchange", crate::EffectKind::WriteFiles),
        RealizationEffect::Clock => ("realization-clock", crate::EffectKind::ReadClock),
        RealizationEffect::Activation => ("realization-activation", crate::EffectKind::ReadFiles),
        RealizationEffect::Environment => ("realization-environment", crate::EffectKind::ReadEnvironment),
        RealizationEffect::Child => ("realization-child", crate::EffectKind::RunProcess),
    };
    debug_assert!(effect_id.starts_with("realization-"));
    debug_assert!(!effect_id.is_empty());
    (effect_id, kind)
}

/// Declare one capability before invoking its adapter.
pub fn realization_effect_plan(effect: RealizationEffect) -> Result<crate::EffectPlan, crate::PlanError> {
    let (effect_id, kind) = realization_effect_authority(effect);
    debug_assert!(effect_id.starts_with("realization-"));
    debug_assert!(!effect_id.is_empty());
    crate::plan_effects(crate::CommandFamily::Realization, &[crate::EffectSpec {
        effect_id,
        kind,
        limit: crate::EffectMeasure::Calls(1),
        expected_output: crate::ExpectedOutput::None,
    }])
}

/// Admit the activation PATH search before checking any candidate file.
pub fn shell_path_search_effect_plan(candidates: u32) -> Result<crate::EffectPlan, crate::PlanError> {
    crate::plan_effects(crate::CommandFamily::Realization, &[crate::EffectSpec {
        effect_id: "shell-path-search",
        kind: crate::EffectKind::ReadFiles,
        limit: crate::EffectMeasure::Items(candidates),
        expected_output: crate::ExpectedOutput::None,
    }])
}

/// Admit explicit activation inputs before checking their presence.
pub fn shell_with_paths_effect_plan(paths: u32) -> Result<crate::EffectPlan, crate::PlanError> {
    crate::plan_effects(crate::CommandFamily::Realization, &[crate::EffectSpec {
        effect_id: "shell-with-paths",
        kind: crate::EffectKind::ReadFiles,
        limit: crate::EffectMeasure::Items(paths),
        expected_output: crate::ExpectedOutput::None,
    }])
}
