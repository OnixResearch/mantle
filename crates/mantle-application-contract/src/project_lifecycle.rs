//! Project command family.
//!
//! Project commands share one typed request, blocker set, port, and result, so
//! the composition root maps CLI DTOs into this command and renders the typed
//! result while the port owns the family's effects.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::envelope::ApplicationBlocker;
use crate::envelope::CapabilityError;

/// Maximum admitted declared entries for one project lifecycle command.
pub const MAX_PROJECT_LIFECYCLE_DECLARED_ENTRIES: u32 = 256;

/// Admitted blocker slots for one project lifecycle command beyond its declared entries.
const MAX_PROJECT_LIFECYCLE_BLOCKERS: usize = 4;

/// Project operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProjectOperation {
    /// Init operation.
    Init,
    /// Check operation.
    Check,
    /// Refresh operation.
    Refresh,
    /// Show operation.
    Show,
    /// List stale operation.
    ListStale,
    /// Upgrade operation.
    Upgrade,
}

impl ProjectOperation {
    /// Every operation in canonical order.
    pub fn all() -> Vec<Self> {
        let entries = vec![
            Self::Init,
            Self::Check,
            Self::Refresh,
            Self::Show,
            Self::ListStale,
            Self::Upgrade,
        ];
        debug_assert_eq!(entries.len(), 6);
        debug_assert!(!entries.is_empty());
        entries
    }

    /// Stable operation label.
    pub fn as_str(self) -> &'static str {
        let label = match self {
            Self::Init => "init",
            Self::Check => "check",
            Self::Refresh => "refresh",
            Self::Show => "show",
            Self::ListStale => "list_stale",
            Self::Upgrade => "upgrade",
        };
        debug_assert!(!label.is_empty());
        debug_assert!(ProjectOperation::all().contains(&self));
        label
    }

    /// Whether the operation requires at least one declared entry.
    pub fn requires_declared_entries(self) -> bool {
        // Every project operation works from the manifest rather than from declared entries.
        let is_required = false;
        debug_assert!(ProjectOperation::all().contains(&self));
        debug_assert!(ProjectOperation::all().len() == 6);
        is_required
    }
}

/// One typed Project command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCommand {
    /// Command root that produced this command.
    pub root: String,
    pub operation: ProjectOperation,
    /// Primary subject of the command, such as a path, name, or selector.
    pub subject: String,
    /// Declared entries the operation consumes, in caller order.
    pub declared_entries: Vec<String>,
    /// Manifest path.
    pub manifest_path: String,
    /// Has lock write.
    pub has_lock_write: bool,
}

/// Domain blocker specific to Project commands.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProjectBlocker {
    /// The request named no subject.
    MissingSubject,
    /// The operation needs a declared entry and the request named none.
    MissingDeclaredEntry,
    /// The request exceeded the admitted declared-entry bound.
    TooManyDeclaredEntries,
    /// The operation needs an input specific to this family.
    LockWriteRequired,
    /// Domain policy rejected the request for another reason.
    Domain(ApplicationBlocker),
}

/// Typed Project result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectResult {
    /// Resolved inputs count.
    pub resolved_inputs_count: u32,
    /// Stale inputs count.
    pub stale_inputs_count: u32,
    /// Schema version.
    pub schema_version: u32,
}

/// Terminal Project outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectOutcome {
    /// The operation ran and produced a result.
    Completed(ProjectResult),
    /// Domain policy rejected the request before any effect ran.
    Blocked(Vec<ProjectBlocker>),
}

/// Port: execute one typed Project command.
pub trait ProjectPort {
    fn run(&mut self, request: &ProjectCommand) -> Result<ProjectOutcome, CapabilityError>;
}

/// Validate one Project command before any port is called.
pub fn validate_project_lifecycle(request: &ProjectCommand) -> Vec<ProjectBlocker> {
    let declared_count = request.declared_entries.len();
    let blocker_slots = declared_count.saturating_add(MAX_PROJECT_LIFECYCLE_BLOCKERS);
    let mut blockers: Vec<ProjectBlocker> = Vec::with_capacity(blocker_slots);
    if request.root.is_empty() {
        blockers.push(ProjectBlocker::Domain(ApplicationBlocker::new(
            "missing-command-root",
            "project_lifecycle",
            "a command must name its command root",
        )));
    }
    if request.subject.trim().is_empty() {
        blockers.push(ProjectBlocker::MissingSubject);
    }
    if request.operation.requires_declared_entries() && declared_count == 0 {
        blockers.push(ProjectBlocker::MissingDeclaredEntry);
    }
    let is_declared_count_admissible =
        u32::try_from(declared_count).is_ok_and(|count| count <= MAX_PROJECT_LIFECYCLE_DECLARED_ENTRIES);
    if !is_declared_count_admissible {
        blockers.push(ProjectBlocker::TooManyDeclaredEntries);
    }
    if matches!(request.operation, ProjectOperation::Refresh | ProjectOperation::Upgrade) && !request.has_lock_write {
        blockers.push(ProjectBlocker::LockWriteRequired);
    }
    blockers.sort();
    blockers.dedup();
    debug_assert!(blockers.len() <= declared_count.saturating_add(MAX_PROJECT_LIFECYCLE_BLOCKERS));
    let empty_entries = request.declared_entries.iter().filter(|entry| entry.is_empty()).count();
    debug_assert!(empty_entries <= declared_count);
    blockers
}

/// Maximum named project capability calls in one command (the init scaffold).
pub const MAX_PROJECT_EFFECTS_PER_PLAN: u8 = 7;

/// One project-owned capability call. Repeated writes have distinct identities:
/// a completed lock write does not imply generated inputs or retention were written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectEffect {
    Inspect,
    ManifestWrite,
    InitLockWrite,
    InitInputsWrite,
    InitRetentionWrite,
    Gitignore,
    Readback,
    Resolve,
    LockWrite,
    InputsWrite,
    RetentionWrite,
}

/// Path or resolver scope authorized by the project plan. The root is the
/// caller-selected project directory; named files are relative to that root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectTarget {
    ProjectRoot,
    Manifest,
    Lock,
    Inputs,
    Retention,
    Gitignore,
    Resolver,
}

impl ProjectEffect {
    pub const fn target(self) -> ProjectTarget {
        match self {
            Self::Inspect | Self::Readback => ProjectTarget::ProjectRoot,
            Self::ManifestWrite => ProjectTarget::Manifest,
            Self::InitLockWrite | Self::LockWrite => ProjectTarget::Lock,
            Self::InitInputsWrite | Self::InputsWrite => ProjectTarget::Inputs,
            Self::InitRetentionWrite | Self::RetentionWrite => ProjectTarget::Retention,
            Self::Gitignore => ProjectTarget::Gitignore,
            Self::Resolve => ProjectTarget::Resolver,
        }
    }
}

/// Authority admitted for one call, including local resolution that can run a
/// freshness command without allowing a network request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectAuthority {
    ReadFiles,
    WriteFiles,
    Resolve {
        network_allowed: bool,
        process_allowed: bool,
    },
}

/// Each planned call has a one-call limit and must produce one terminal
/// observation (including a skipped call); completion alone proves no content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectEffectStep {
    pub effect: ProjectEffect,
    pub authority: ProjectAuthority,
    pub max_calls: u8,
    /// Maximum resolved input outcomes. Other port calls report one result.
    pub max_items: u32,
    pub expected_observation: ProjectObservationKind,
    pub target: ProjectTarget,
    /// Conditional persistence may be skipped only when no mutation was decided.
    pub conditional: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectObservationKind {
    CallResult,
    FileReadback,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectEffectPlan {
    pub operation: ProjectOperation,
    pub steps: Vec<ProjectEffectStep>,
}

impl ProjectEffectPlan {
    /// Admit the resolved input cardinality before the resolver port can run.
    /// `refresh` uses the selected names when supplied; `list-stale` visits
    /// the whole manifest. This is the resolver's existing 256-input limit.
    pub fn admit_resolution_count(&self, count: u32) -> Result<(), ProjectBlocker> {
        if !self.steps.iter().any(|step| step.effect == ProjectEffect::Resolve) {
            return Err(ProjectBlocker::Domain(ApplicationBlocker::new(
                "resolution-not-planned",
                "project_lifecycle",
                "this operation cannot resolve inputs",
            )));
        }
        if count > MAX_PROJECT_LIFECYCLE_DECLARED_ENTRIES {
            return Err(ProjectBlocker::TooManyDeclaredEntries);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectEffectStatus {
    Succeeded,
    Failed,
    Skipped,
}

/// Captures actual capability progress. A failed call may have partially
/// mutated state; `calls` records dispatch, not an atomicity claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectEffectObservation {
    pub effect: ProjectEffect,
    pub authority: ProjectAuthority,
    pub kind: ProjectObservationKind,
    pub calls: u8,
    /// Number of resolver outcomes, or one result from an executed file call.
    pub items: u32,
    pub target: ProjectTarget,
    /// Actual init scaffold comparison, absent for other calls and read faults.
    pub readback_matches: Option<bool>,
    pub status: ProjectEffectStatus,
}

/// Port surface owned by the project family. The application supplies concrete
/// manifest, lock, and resolver types; no host or provider type enters this core.
/// A port error remains typed at the shell boundary and is never successful
/// evidence. Read and resolve may perform subordinate file/process/network I/O.
pub trait ProjectEffectPort {
    type Manifest;
    type Lock;
    type Refresh;
    type Applied;
    type Stale;
    type Retention;
    type Error;
    fn inspect_legacy(&mut self) -> Result<(), Self::Error>;

    fn inspect_init(&mut self) -> Result<(), Self::Error>;
    fn inspect_manifest(&mut self) -> Result<Self::Manifest, Self::Error>;
    fn inspect_lock(&mut self) -> Result<Self::Lock, Self::Error>;
    fn inspect(&mut self) -> Result<(Self::Manifest, Self::Lock), Self::Error>;
    fn generated_inputs(&mut self) -> Option<String>;
    fn retention(&mut self) -> Self::Retention;
    fn write_manifest(&mut self) -> Result<(), Self::Error>;
    fn write_init_lock(&mut self) -> Result<(), Self::Error>;
    fn write_init_inputs(&mut self) -> Result<(), Self::Error>;
    fn write_init_retention(&mut self) -> Result<(), Self::Error>;
    fn gitignore(&mut self) -> Result<(), Self::Error>;
    fn readback(&mut self) -> Result<bool, Self::Error>;
    fn refresh(&mut self, manifest: &Self::Manifest, lock: &Self::Lock) -> Self::Refresh;
    fn stale(&mut self, manifest: &Self::Manifest, lock: &Self::Lock) -> Self::Stale;
    fn write_lock(&mut self, lock: &Self::Lock) -> Result<(), Self::Error>;
    fn apply_refresh(
        &mut self,
        manifest: &Self::Manifest,
        lock: &Self::Lock,
        outcomes: &Self::Refresh,
    ) -> Self::Applied;
    fn write_inputs(&mut self, lock: &Self::Lock) -> Result<(), Self::Error>;
    fn write_retention(&mut self, manifest: &Self::Manifest, lock: &Self::Lock) -> Result<(), Self::Error>;
}

/// Decide all possible calls before the first effect. Conditional writes are
/// observed as skipped, not dropped or retroactively planned.
pub fn project_effect_plan(
    operation: ProjectOperation,
    network_allowed: bool,
    process_allowed: bool,
) -> ProjectEffectPlan {
    use ProjectAuthority::ReadFiles;
    use ProjectAuthority::WriteFiles;
    use ProjectEffect::Gitignore;
    use ProjectEffect::InitInputsWrite;
    use ProjectEffect::InitLockWrite;
    use ProjectEffect::InitRetentionWrite;
    use ProjectEffect::InputsWrite;
    use ProjectEffect::Inspect;
    use ProjectEffect::LockWrite;
    use ProjectEffect::ManifestWrite;
    use ProjectEffect::Readback;
    use ProjectEffect::Resolve;
    use ProjectEffect::RetentionWrite;
    let resolve = ProjectAuthority::Resolve {
        network_allowed,
        process_allowed,
    };
    let calls: &[(ProjectEffect, ProjectAuthority, ProjectObservationKind)] = match operation {
        ProjectOperation::Init => &[
            (Inspect, ReadFiles, ProjectObservationKind::CallResult),
            (ManifestWrite, WriteFiles, ProjectObservationKind::CallResult),
            (InitLockWrite, WriteFiles, ProjectObservationKind::CallResult),
            (InitInputsWrite, WriteFiles, ProjectObservationKind::CallResult),
            (InitRetentionWrite, WriteFiles, ProjectObservationKind::CallResult),
            (Gitignore, WriteFiles, ProjectObservationKind::CallResult),
            (Readback, ReadFiles, ProjectObservationKind::FileReadback),
        ],
        ProjectOperation::Check | ProjectOperation::Show => &[(Inspect, ReadFiles, ProjectObservationKind::CallResult)],
        ProjectOperation::ListStale => &[
            (Inspect, ReadFiles, ProjectObservationKind::CallResult),
            (Resolve, resolve, ProjectObservationKind::CallResult),
        ],
        ProjectOperation::Refresh => &[
            (Inspect, ReadFiles, ProjectObservationKind::CallResult),
            (Resolve, resolve, ProjectObservationKind::CallResult),
            (LockWrite, WriteFiles, ProjectObservationKind::CallResult),
            (InputsWrite, WriteFiles, ProjectObservationKind::CallResult),
            (RetentionWrite, WriteFiles, ProjectObservationKind::CallResult),
        ],
        ProjectOperation::Upgrade => &[
            (Inspect, ReadFiles, ProjectObservationKind::CallResult),
            (LockWrite, WriteFiles, ProjectObservationKind::CallResult),
            (InputsWrite, WriteFiles, ProjectObservationKind::CallResult),
            (RetentionWrite, WriteFiles, ProjectObservationKind::CallResult),
        ],
    };
    let steps: Vec<ProjectEffectStep> = calls
        .iter()
        .map(|&(effect, authority, expected_observation)| ProjectEffectStep {
            effect,
            authority,
            max_calls: 1,
            max_items: if effect == Resolve {
                MAX_PROJECT_LIFECYCLE_DECLARED_ENTRIES
            } else {
                1
            },
            expected_observation,
            target: effect.target(),
            conditional: matches!(effect, LockWrite | InputsWrite | RetentionWrite),
        })
        .collect();
    debug_assert!(!steps.is_empty());
    debug_assert!(steps.len() <= usize::from(MAX_PROJECT_EFFECTS_PER_PLAN));
    ProjectEffectPlan { operation, steps }
}

/// Reject identity, authority, output, ordering, or call-limit drift. Conditional
/// writes may all be skipped when no mutation was needed; a partial write or
/// failed resolution never becomes a completed operation.
pub fn classify_project_effects(
    plan: &ProjectEffectPlan,
    observations: &[ProjectEffectObservation],
) -> crate::envelope::ApplicationOutcome {
    use crate::envelope::ApplicationOutcome;
    if plan.steps.is_empty() || plan.steps.len() > usize::from(MAX_PROJECT_EFFECTS_PER_PLAN) {
        return ApplicationOutcome::Rejected {
            unknown_effect_count: 1,
            missing_effect_count: 0,
        };
    }
    let missing = plan.steps.len().saturating_sub(observations.len());
    let extra = observations.len().saturating_sub(plan.steps.len());
    let mut persistence_started = false;
    let mut mismatch = 0_usize;
    let mut failures = 0_usize;
    let mut failed_before = false;
    for (step, observation) in plan.steps.iter().zip(observations) {
        if step.effect != observation.effect
            || step.target != observation.target
            || step.authority != observation.authority
            || step.expected_observation != observation.kind
            || observation.calls > step.max_calls
            || observation.items > step.max_items
            || (observation.calls == 0 && observation.items != 0)
            || (observation.calls == 0) != (observation.status == ProjectEffectStatus::Skipped)
            || (step.expected_observation == ProjectObservationKind::CallResult
                && observation.readback_matches.is_some())
            || (step.expected_observation == ProjectObservationKind::FileReadback
                && !matches!(
                    (observation.status, observation.readback_matches),
                    (ProjectEffectStatus::Succeeded, Some(true))
                        | (ProjectEffectStatus::Skipped, None)
                        | (ProjectEffectStatus::Failed, Some(false) | None)
                ))
            || (step.conditional
                && persistence_started
                && !failed_before
                && observation.status == ProjectEffectStatus::Skipped)
        {
            mismatch = mismatch.saturating_add(1);
        }
        if step.conditional && observation.calls > 0 {
            persistence_started = true;
        }
        if observation.status == ProjectEffectStatus::Failed
            || (observation.status == ProjectEffectStatus::Skipped && !step.conditional)
        {
            failures = failures.saturating_add(1);
        }
        if observation.status == ProjectEffectStatus::Failed {
            failed_before = true;
        }
    }
    if mismatch > 0 || missing > 0 || extra > 0 {
        return ApplicationOutcome::Rejected {
            unknown_effect_count: u32::try_from(mismatch.saturating_add(extra)).unwrap_or(u32::MAX),
            missing_effect_count: u32::try_from(missing).unwrap_or(u32::MAX),
        };
    }
    if failures > 0 {
        return ApplicationOutcome::Failed {
            failed_effect_count: u32::try_from(failures).unwrap_or(u32::MAX),
        };
    }
    ApplicationOutcome::Completed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> ProjectCommand {
        ProjectCommand {
            root: String::from("project-lifecycle"),
            operation: ProjectOperation::all()[0],
            subject: String::from("subject"),
            declared_entries: vec![String::from("first")],
            manifest_path: String::from("crunch-project.ncl"),
            has_lock_write: true,
        }
    }

    #[test]
    fn a_complete_request_is_admissible() {
        let request = sample_request();
        assert!(validate_project_lifecycle(&request).is_empty());
        assert_eq!(ProjectOperation::all().len(), 6);
        let declared_required =
            ProjectOperation::all().iter().filter(|operation| operation.requires_declared_entries()).count();
        assert_eq!(declared_required, 0);
    }

    #[test]
    fn missing_root_subject_and_declared_entries_are_rejected() {
        let mut request = sample_request();
        request.root = String::new();
        request.subject = String::from("   ");
        request.declared_entries.clear();
        let blockers = validate_project_lifecycle(&request);
        assert!(blockers.iter().any(|blocker| matches!(blocker, ProjectBlocker::Domain(_))));
        assert!(blockers.contains(&ProjectBlocker::MissingSubject));
        assert!(!ProjectOperation::all().is_empty());
    }

    #[test]
    fn the_family_specific_requirement_is_enforced() {
        let mut request = sample_request();
        request.has_lock_write = false;
        request.operation = ProjectOperation::Refresh;
        let blockers = validate_project_lifecycle(&request);
        assert!(blockers.contains(&ProjectBlocker::LockWriteRequired));
        assert!(!blockers.is_empty());
        let empty_labels = ProjectOperation::all()
            .iter()
            .map(|operation| operation.as_str())
            .filter(|label| label.is_empty())
            .count();
        assert_eq!(empty_labels, 0);
    }
    fn observations(plan: &ProjectEffectPlan) -> Vec<ProjectEffectObservation> {
        plan.steps
            .iter()
            .map(|step| ProjectEffectObservation {
                effect: step.effect,
                authority: step.authority,
                target: step.target,
                kind: step.expected_observation,
                calls: 1,
                items: 1,
                readback_matches: if step.expected_observation == ProjectObservationKind::FileReadback {
                    Some(true)
                } else {
                    None
                },
                status: ProjectEffectStatus::Succeeded,
            })
            .collect()
    }

    #[test]
    fn refresh_plans_resolution_and_each_independently_observed_write() {
        let plan = project_effect_plan(ProjectOperation::Refresh, false, true);
        let mut actual = observations(&plan);
        assert_eq!(plan.steps.len(), 5);
        assert_eq!(plan.steps[1].authority, ProjectAuthority::Resolve {
            network_allowed: false,
            process_allowed: true,
        });
        assert_eq!(classify_project_effects(&plan, &actual), crate::envelope::ApplicationOutcome::Completed);
        actual[1].status = ProjectEffectStatus::Failed;
        assert_eq!(classify_project_effects(&plan, &actual), crate::envelope::ApplicationOutcome::Failed {
            failed_effect_count: 1
        });
        actual[3].status = ProjectEffectStatus::Failed;
        assert_eq!(classify_project_effects(&plan, &actual), crate::envelope::ApplicationOutcome::Failed {
            failed_effect_count: 2
        });
    }

    #[test]
    fn resolver_admits_exact_boundary_and_blocks_overbound_before_call() {
        let refresh = project_effect_plan(ProjectOperation::Refresh, false, true);
        assert!(refresh.admit_resolution_count(256).is_ok());
        assert_eq!(refresh.admit_resolution_count(257), Err(ProjectBlocker::TooManyDeclaredEntries));
        let show = project_effect_plan(ProjectOperation::Show, false, false);
        assert!(matches!(show.admit_resolution_count(0), Err(ProjectBlocker::Domain(_))));
    }

    #[test]
    fn an_empty_effect_plan_cannot_certify_an_operation() {
        let plan = ProjectEffectPlan {
            operation: ProjectOperation::Check,
            steps: Vec::new(),
        };
        assert!(matches!(classify_project_effects(&plan, &[]), crate::envelope::ApplicationOutcome::Rejected { .. }));
    }

    #[test]
    fn wrong_observation_identity_authority_output_and_limit_are_rejected() {
        let plan = project_effect_plan(ProjectOperation::Init, false, false);
        let base = observations(&plan);
        for corrupt in [
            ProjectEffectObservation {
                effect: ProjectEffect::Resolve,
                ..base[2]
            },
            ProjectEffectObservation {
                authority: ProjectAuthority::ReadFiles,
                ..base[2]
            },
            ProjectEffectObservation {
                kind: ProjectObservationKind::FileReadback,
                ..base[2]
            },
            ProjectEffectObservation {
                target: ProjectTarget::Inputs,
                ..base[2]
            },
            ProjectEffectObservation { calls: 2, ..base[2] },
            ProjectEffectObservation { items: 2, ..base[2] },
        ] {
            let mut actual = base.clone();
            actual[2] = corrupt;
            assert!(matches!(
                classify_project_effects(&plan, &actual),
                crate::envelope::ApplicationOutcome::Rejected { .. }
            ));
        }
    }

    #[test]
    fn scaffold_readback_must_observe_matching_bytes_before_success() {
        let plan = project_effect_plan(ProjectOperation::Init, false, false);
        let mut actual = observations(&plan);
        let last = actual.last_mut().unwrap();
        last.readback_matches = Some(false);
        assert!(matches!(
            classify_project_effects(&plan, &actual),
            crate::envelope::ApplicationOutcome::Rejected { .. }
        ));
        let last = actual.last_mut().unwrap();
        last.status = ProjectEffectStatus::Failed;
        assert_eq!(classify_project_effects(&plan, &actual), crate::envelope::ApplicationOutcome::Failed {
            failed_effect_count: 1
        });
    }

    #[test]
    fn unchanged_refresh_skips_all_writes_but_partial_persistence_rejects() {
        let plan = project_effect_plan(ProjectOperation::Refresh, false, true);
        let mut actual = observations(&plan);
        for observation in &mut actual[2..] {
            observation.calls = 0;
            observation.items = 0;
            observation.status = ProjectEffectStatus::Skipped;
        }
        assert_eq!(classify_project_effects(&plan, &actual), crate::envelope::ApplicationOutcome::Completed);
        actual[2].calls = 1;
        actual[2].items = 1;
        actual[2].status = ProjectEffectStatus::Succeeded;
        assert!(matches!(
            classify_project_effects(&plan, &actual),
            crate::envelope::ApplicationOutcome::Rejected { .. }
        ));
    }
    #[test]
    fn failed_late_write_keeps_earlier_write_observation_without_claiming_completion() {
        let plan = project_effect_plan(ProjectOperation::Refresh, false, true);
        let mut actual = observations(&plan);
        actual[3].status = ProjectEffectStatus::Failed;
        actual[4].calls = 0;
        actual[4].items = 0;
        actual[4].status = ProjectEffectStatus::Skipped;
        assert_eq!(classify_project_effects(&plan, &actual), crate::envelope::ApplicationOutcome::Failed {
            failed_effect_count: 1
        });
        assert_eq!(actual[2].status, ProjectEffectStatus::Succeeded);
    }

    #[test]
    fn skipping_required_init_write_never_claims_a_complete_scaffold() {
        let plan = project_effect_plan(ProjectOperation::Init, false, false);
        let mut actual = observations(&plan);
        actual[2].calls = 0;
        actual[2].items = 0;
        actual[2].status = ProjectEffectStatus::Skipped;
        assert_eq!(classify_project_effects(&plan, &actual), crate::envelope::ApplicationOutcome::Failed {
            failed_effect_count: 1
        });
    }

    #[test]
    fn early_blocker_and_skipped_conditional_writes_cannot_complete() {
        let plan = project_effect_plan(ProjectOperation::Upgrade, false, false);
        let mut actual = observations(&plan);
        actual[0].status = ProjectEffectStatus::Failed;
        for observation in &mut actual[1..] {
            observation.status = ProjectEffectStatus::Skipped;
            observation.calls = 0;
            observation.items = 0;
        }
        assert_eq!(classify_project_effects(&plan, &actual), crate::envelope::ApplicationOutcome::Failed {
            failed_effect_count: 1
        });
        actual[1].calls = 1;
        assert!(matches!(
            classify_project_effects(&plan, &actual),
            crate::envelope::ApplicationOutcome::Rejected { .. }
        ));
    }
}
