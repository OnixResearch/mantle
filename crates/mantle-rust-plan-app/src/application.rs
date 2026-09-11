//! Rust-plan orchestration over application-owned ports.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use mantle_rust_plan_core::BuildProfile;
use mantle_rust_plan_core::MAX_UNITS;
use mantle_rust_plan_core::PlanExecutionOutcome;
use mantle_rust_plan_core::PlanOutcome;
use mantle_rust_plan_core::PlanReceiptPreimage;
use mantle_rust_plan_core::PlanRequest;
use mantle_rust_plan_core::RustPlan;
use mantle_rust_plan_core::UnitEffect;
use mantle_rust_plan_core::UnitLimitFacts;
use mantle_rust_plan_core::UnitObservation;
use mantle_rust_plan_core::UnitObservationStatus;
use mantle_rust_plan_core::build_receipt_preimage;
use mantle_rust_plan_core::classify_plan_observations;
use mantle_rust_plan_core::plan_rust_units;

use crate::ports::AdapterError;
use crate::ports::CacheLookup;
use crate::ports::CacheLookupRequest;
use crate::ports::CargoOracleCapture;
use crate::ports::CompilerFacts;
use crate::ports::CompilerInspection;
use crate::ports::CompilerInspectionRequest;
use crate::ports::OracleCaptureRequest;
use crate::ports::OracleFacts;
use crate::ports::RustCacheAccess;
use crate::ports::UnitExecutor;
use crate::ports::WorkspaceFactsRequest;
use crate::ports::WorkspaceFactsSource;
use crate::ports::WorkspaceFactsView;

/// What happened to one unit during execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitDisposition {
    /// The unit's outputs were already cached under the planned identity.
    CacheHit,
    /// The unit was executed and observed.
    Executed,
}

/// Cache accounting for one run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheDisposition {
    /// Every executed unit missed the cache.
    AllMissed,
    /// At least one unit was served from the cache; executed units still ran.
    PartiallyHit,
    /// Every unit was served from the cache; no unit executed.
    AllHit,
}

/// One executed plan with its observations and evidence links.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedPlan {
    pub plan: RustPlan,
    pub receipt_preimage: Option<PlanReceiptPreimage>,
    pub observations: Vec<UnitObservation>,
    pub execution: PlanExecutionOutcome,
    pub cache: CacheDisposition,
    pub dispositions: Vec<UnitDisposition>,
}

/// Terminal application outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationOutcome {
    /// Planning rejected the request; nothing executed.
    Blocked {
        blockers: Vec<mantle_rust_plan_core::PlanBlocker>,
    },
    /// The plan executed and its observations classified.
    Executed(Box<ExecutedPlan>),
}

/// Rust-plan application wired to explicit capability ports.
pub struct RustPlanApplication<W, O, C, X, E> {
    workspace: W,
    oracle: O,
    compiler: C,
    cache: X,
    executor: E,
    limits: UnitLimitFacts,
}

impl<W, O, C, X, E> RustPlanApplication<W, O, C, X, E>
where
    W: WorkspaceFactsSource,
    O: CargoOracleCapture,
    C: CompilerInspection,
    X: RustCacheAccess,
    E: UnitExecutor,
{
    /// Build an application over explicit ports.
    pub fn new(workspace: W, oracle: O, compiler: C, cache: X, executor: E) -> Self {
        Self {
            workspace,
            oracle,
            compiler,
            cache,
            executor,
            limits: UnitLimitFacts { max_units: MAX_UNITS },
        }
    }

    /// Override the declared unit limit.
    pub fn with_unit_limit(mut self, limits: UnitLimitFacts) -> Self {
        self.limits = limits;
        self
    }

    /// Plan and execute one request through the ports.
    ///
    /// Capability failures surface as typed `AdapterError` values; planning
    /// blockers surface as `ApplicationOutcome::Blocked` and perform no work.
    pub fn run(
        &mut self,
        roots: Vec<String>,
        profile: BuildProfile,
        include_oracle: bool,
    ) -> Result<ApplicationOutcome, AdapterError> {
        let facts = self.load_facts(&roots, profile, include_oracle)?;
        let plan = plan_rust_units(&PlanRequest {
            roots: roots.clone(),
            packages: facts.packages,
            feature_requests: facts.feature_requests,
            profile,
            limits: self.limits,
        });
        if plan.outcome == PlanOutcome::Blocked {
            return Ok(ApplicationOutcome::Blocked {
                blockers: plan.blockers,
            });
        }
        self.execute_plan(plan)
    }

    /// Load workspace facts plus optional oracle and compiler observations.
    fn load_facts(
        &mut self,
        roots: &[String],
        profile: BuildProfile,
        include_oracle: bool,
    ) -> Result<WorkspaceFactsView, AdapterError> {
        let mut facts = self.workspace.load_workspace_facts(&WorkspaceFactsRequest {
            roots: roots.to_vec(),
            profile,
        })?;
        if include_oracle {
            let _oracle: OracleFacts = self.oracle.capture_oracle(&OracleCaptureRequest {
                roots: roots.to_vec(),
                profile,
            })?;
        }
        let _compiler: CompilerFacts = self.compiler.inspect_compiler(&CompilerInspectionRequest { profile })?;
        facts.feature_requests.sort_by(|left, right| left.package.cmp(&right.package));
        debug_assert!(facts.feature_requests.windows(2).all(|pair| pair[0].package != pair[1].package));
        Ok(facts)
    }

    /// Execute every planned effect, consulting the cache first.
    fn execute_plan(&mut self, plan: RustPlan) -> Result<ApplicationOutcome, AdapterError> {
        let mut observations: Vec<UnitObservation> = Vec::with_capacity(plan.effects.len());
        let mut dispositions: Vec<UnitDisposition> = Vec::with_capacity(plan.effects.len());
        let mut cache_hits: u32 = 0;
        for effect in &plan.effects {
            match self.cache_hit(effect)? {
                Some(()) => {
                    cache_hits = cache_hits.saturating_add(1);
                    dispositions.push(UnitDisposition::CacheHit);
                    observations.push(cached_observation(effect));
                }
                None => {
                    let observation = self.executor.execute_unit(effect)?;
                    dispositions.push(UnitDisposition::Executed);
                    observations.push(observation);
                }
            }
        }
        let execution = classify_plan_observations(&plan, &observations);
        let cache = cache_disposition(cache_hits, plan.effects.len());
        let receipt_preimage = build_receipt_preimage(&plan);
        debug_assert_eq!(observations.len(), plan.effects.len());
        debug_assert_eq!(dispositions.len(), plan.effects.len());
        Ok(ApplicationOutcome::Executed(Box::new(ExecutedPlan {
            plan,
            receipt_preimage,
            observations,
            execution,
            cache,
            dispositions,
        })))
    }

    fn cache_hit(&mut self, effect: &UnitEffect) -> Result<Option<()>, AdapterError> {
        let lookup = self.cache.lookup_unit(&CacheLookupRequest {
            unit_id: effect.unit_id.clone(),
            unit_blake3: first_input_identity(effect),
        })?;
        match lookup {
            CacheLookup::Cached { output_identity } => {
                debug_assert!(!output_identity.is_empty());
                Ok(Some(()))
            }
            CacheLookup::NotCached => Ok(None),
        }
    }
}

/// Observation recorded for a unit served from the cache.
fn cached_observation(effect: &UnitEffect) -> UnitObservation {
    debug_assert!(!effect.effect_id.0.is_empty());
    UnitObservation {
        effect_id: effect.effect_id.clone(),
        unit_id: effect.unit_id.clone(),
        status: UnitObservationStatus::Succeeded,
        exit_code: Some(0),
        diagnostics_code: Some(String::from("cache-hit")),
    }
}

fn first_input_identity(effect: &UnitEffect) -> mantle_rust_plan_core::Blake3Digest {
    match effect.input_identities.first() {
        Some(identity) => identity.clone(),
        None => mantle_rust_plan_core::Blake3Digest::from_slice(b"missing-input-identity"),
    }
}

fn cache_disposition(cache_hits: u32, effect_count: usize) -> CacheDisposition {
    let Ok(total) = u32::try_from(effect_count) else {
        // A plan wider than the disposition domain cannot report hits.
        return CacheDisposition::AllMissed;
    };
    if cache_hits == 0 {
        CacheDisposition::AllMissed
    } else if cache_hits >= total {
        CacheDisposition::AllHit
    } else {
        CacheDisposition::PartiallyHit
    }
}
