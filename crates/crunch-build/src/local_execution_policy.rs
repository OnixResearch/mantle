//! Pure local sandbox execution-budget and staged resource-policy kernel.
//!
//! This module contains zero clocks, zero process/cgroup syscalls, and zero
//! derivation-identity dependencies. Runtime enforcement lives in
//! `snix_build::buildservice::BubblewrapBuildService` and `snix_build::bwrap`.

use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

pub const LOCAL_EXECUTION_POLICY_SCHEMA: &str = "mantle-local-execution-policy-v1";
pub const DEFAULT_WALL_DEADLINE_SECS: u64 = 3_600;
pub const MIN_WALL_DEADLINE_SECS: u64 = 1;
pub const MAX_WALL_DEADLINE_SECS: u64 = 86_400;
pub const DEFAULT_SHUTDOWN_GRACE_MS: u64 = 2_000;
pub const MIN_SHUTDOWN_GRACE_MS: u64 = 20;
pub const MAX_SHUTDOWN_GRACE_MS: u64 = 60_000;
pub const MIN_SILENCE_TIMEOUT_SECS: u64 = 1;
pub const MAX_SILENCE_TIMEOUT_SECS: u64 = 86_400;
pub const MIN_MEMORY_LIMIT_BYTES: u64 = 4_194_304;
pub const MAX_MEMORY_LIMIT_BYTES: u64 = 1_099_511_627_776;
pub const MIN_PIDS_LIMIT_COUNT: u32 = 4;
pub const MAX_PIDS_LIMIT_COUNT: u32 = 65_536;
pub const MIN_SCRATCH_LIMIT_BYTES: u64 = 1_048_576;
pub const MAX_SCRATCH_LIMIT_BYTES: u64 = 1_099_511_627_776;
const MS_PER_SEC: u64 = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalWallDeadlinePolicy {
    Bounded { wall_deadline_secs: u64 },
    Unlimited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalSilenceBudgetPolicy {
    Disabled,
    Bounded { silence_timeout_secs: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalEnforcementMode {
    Disabled,
    BestEffort,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalSwapLimitPolicy {
    HostDefault,
    Zero,
    Bounded { swap_limit_bytes: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalHelperScopePolicy {
    RequireContained,
    AllowInProcessPassiveDatagram,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalResourceBudgetPolicy {
    pub cgroup_root: Option<PathBuf>,
    pub memory_mode: LocalEnforcementMode,
    pub memory_limit_bytes: Option<u64>,
    pub swap_policy: LocalSwapLimitPolicy,
    pub pids_mode: LocalEnforcementMode,
    pub pids_limit_count: Option<u32>,
    pub scratch_mode: LocalEnforcementMode,
    pub scratch_limit_bytes: Option<u64>,
    pub helper_scope_policy: LocalHelperScopePolicy,
}

impl Default for LocalResourceBudgetPolicy {
    fn default() -> Self {
        Self::disabled()
    }
}

impl LocalResourceBudgetPolicy {
    pub const fn disabled() -> Self {
        Self {
            cgroup_root: None,
            memory_mode: LocalEnforcementMode::Disabled,
            memory_limit_bytes: None,
            swap_policy: LocalSwapLimitPolicy::Zero,
            pids_mode: LocalEnforcementMode::Disabled,
            pids_limit_count: None,
            scratch_mode: LocalEnforcementMode::Disabled,
            scratch_limit_bytes: None,
            helper_scope_policy: LocalHelperScopePolicy::RequireContained,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalExecutionPolicy {
    pub schema: String,
    pub wall_deadline: LocalWallDeadlinePolicy,
    pub silence_budget: LocalSilenceBudgetPolicy,
    pub shutdown_grace_ms: u64,
    pub resources: LocalResourceBudgetPolicy,
}

impl Default for LocalExecutionPolicy {
    fn default() -> Self {
        Self::standard()
    }
}

impl LocalExecutionPolicy {
    pub fn standard() -> Self {
        Self {
            schema: LOCAL_EXECUTION_POLICY_SCHEMA.to_string(),
            wall_deadline: LocalWallDeadlinePolicy::Bounded {
                wall_deadline_secs: DEFAULT_WALL_DEADLINE_SECS,
            },
            silence_budget: LocalSilenceBudgetPolicy::Disabled,
            shutdown_grace_ms: DEFAULT_SHUTDOWN_GRACE_MS,
            resources: LocalResourceBudgetPolicy::disabled(),
        }
    }

    pub fn unlimited() -> Self {
        Self {
            schema: LOCAL_EXECUTION_POLICY_SCHEMA.to_string(),
            wall_deadline: LocalWallDeadlinePolicy::Unlimited,
            silence_budget: LocalSilenceBudgetPolicy::Disabled,
            shutdown_grace_ms: DEFAULT_SHUTDOWN_GRACE_MS,
            resources: LocalResourceBudgetPolicy::disabled(),
        }
    }

    pub fn validate(&self) -> Result<(), LocalExecutionPolicyError> {
        validate_schema_and_deadlines(self)?;
        validate_resource_limits(&self.resources)?;
        debug_assert_eq!(self.schema, LOCAL_EXECUTION_POLICY_SCHEMA);
        debug_assert!(self.shutdown_grace_ms >= MIN_SHUTDOWN_GRACE_MS);
        Ok(())
    }

    pub fn to_snix_execution_policy(
        &self,
    ) -> Result<snix_build::buildservice::SandboxExecutionPolicy, LocalExecutionPolicyError> {
        self.validate()?;
        let wall_deadline = convert_wall_deadline(self.wall_deadline)?;
        let silence_budget = convert_silence_budget(self.silence_budget)?;
        let resources = convert_resource_policy(&self.resources)?;
        debug_assert!(self.shutdown_grace_ms <= MAX_SHUTDOWN_GRACE_MS);
        debug_assert!(self.shutdown_grace_ms >= MIN_SHUTDOWN_GRACE_MS);
        Ok(snix_build::buildservice::SandboxExecutionPolicy {
            wall_deadline,
            silence_budget,
            shutdown_grace_ms: self.shutdown_grace_ms,
            resources,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LocalExecutionPolicyError {
    #[error("unsupported local execution policy schema: {schema}")]
    UnsupportedSchema { schema: String },
    #[error("local execution policy field '{field}' is out of bounds: {detail}")]
    InvalidBound { field: &'static str, detail: String },
    #[error("local execution policy resource '{field}' configuration is invalid: {detail}")]
    InvalidResourceConfig { field: &'static str, detail: String },
}

fn validate_schema_and_deadlines(policy: &LocalExecutionPolicy) -> Result<(), LocalExecutionPolicyError> {
    if policy.schema != LOCAL_EXECUTION_POLICY_SCHEMA {
        return Err(LocalExecutionPolicyError::UnsupportedSchema {
            schema: policy.schema.clone(),
        });
    }
    match policy.wall_deadline {
        LocalWallDeadlinePolicy::Bounded { wall_deadline_secs } => {
            if wall_deadline_secs < MIN_WALL_DEADLINE_SECS {
                return Err(LocalExecutionPolicyError::InvalidBound {
                    field: "wall_deadline_secs",
                    detail: "must be >= 1; use Unlimited to opt out".to_string(),
                });
            }
            if wall_deadline_secs > MAX_WALL_DEADLINE_SECS {
                return Err(LocalExecutionPolicyError::InvalidBound {
                    field: "wall_deadline_secs",
                    detail: format!("exceeds maximum {MAX_WALL_DEADLINE_SECS}"),
                });
            }
        }
        LocalWallDeadlinePolicy::Unlimited => {}
    }
    match policy.silence_budget {
        LocalSilenceBudgetPolicy::Disabled => {}
        LocalSilenceBudgetPolicy::Bounded { silence_timeout_secs } => {
            if silence_timeout_secs < MIN_SILENCE_TIMEOUT_SECS {
                return Err(LocalExecutionPolicyError::InvalidBound {
                    field: "silence_timeout_secs",
                    detail: "must be >= 1; use Disabled to opt out".to_string(),
                });
            }
            if silence_timeout_secs > MAX_SILENCE_TIMEOUT_SECS {
                return Err(LocalExecutionPolicyError::InvalidBound {
                    field: "silence_timeout_secs",
                    detail: format!("exceeds maximum {MAX_SILENCE_TIMEOUT_SECS}"),
                });
            }
        }
    }
    validate_shutdown_grace(policy.shutdown_grace_ms)?;
    debug_assert_eq!(policy.schema, LOCAL_EXECUTION_POLICY_SCHEMA);
    debug_assert!(policy.shutdown_grace_ms <= MAX_SHUTDOWN_GRACE_MS);
    Ok(())
}

fn validate_shutdown_grace(shutdown_grace_ms: u64) -> Result<(), LocalExecutionPolicyError> {
    if shutdown_grace_ms < MIN_SHUTDOWN_GRACE_MS {
        return Err(LocalExecutionPolicyError::InvalidBound {
            field: "shutdown_grace_ms",
            detail: format!("must be >= {MIN_SHUTDOWN_GRACE_MS}"),
        });
    }
    if shutdown_grace_ms > MAX_SHUTDOWN_GRACE_MS {
        return Err(LocalExecutionPolicyError::InvalidBound {
            field: "shutdown_grace_ms",
            detail: format!("must be <= {MAX_SHUTDOWN_GRACE_MS}"),
        });
    }
    debug_assert!(shutdown_grace_ms >= MIN_SHUTDOWN_GRACE_MS);
    debug_assert!(shutdown_grace_ms <= MAX_SHUTDOWN_GRACE_MS);
    Ok(())
}

fn validate_resource_limits(resources: &LocalResourceBudgetPolicy) -> Result<(), LocalExecutionPolicyError> {
    validate_memory_limit(resources)?;
    validate_pids_limit(resources)?;
    validate_scratch_limit(resources)?;
    debug_assert!(resources.memory_limit_bytes.is_none_or(|bytes| bytes <= MAX_MEMORY_LIMIT_BYTES));
    debug_assert!(resources.pids_limit_count.is_none_or(|count| count <= MAX_PIDS_LIMIT_COUNT));
    Ok(())
}

fn validate_memory_limit(resources: &LocalResourceBudgetPolicy) -> Result<(), LocalExecutionPolicyError> {
    match resources.memory_mode {
        LocalEnforcementMode::Disabled => {}
        LocalEnforcementMode::BestEffort | LocalEnforcementMode::Required => {
            if resources.memory_limit_bytes.is_none() {
                return Err(LocalExecutionPolicyError::InvalidResourceConfig {
                    field: "memory_limit_bytes",
                    detail: "must be present when memory enforcement is enabled".to_string(),
                });
            }
        }
    }
    match resources.memory_limit_bytes {
        None => {}
        Some(limit_bytes) => {
            if limit_bytes < MIN_MEMORY_LIMIT_BYTES {
                return Err(LocalExecutionPolicyError::InvalidBound {
                    field: "memory_limit_bytes",
                    detail: format!("must be >= {MIN_MEMORY_LIMIT_BYTES}"),
                });
            }
            if limit_bytes > MAX_MEMORY_LIMIT_BYTES {
                return Err(LocalExecutionPolicyError::InvalidBound {
                    field: "memory_limit_bytes",
                    detail: format!("must be <= {MAX_MEMORY_LIMIT_BYTES}"),
                });
            }
        }
    }
    match resources.swap_policy {
        LocalSwapLimitPolicy::HostDefault | LocalSwapLimitPolicy::Zero => {}
        LocalSwapLimitPolicy::Bounded { swap_limit_bytes } => {
            if swap_limit_bytes > MAX_MEMORY_LIMIT_BYTES {
                return Err(LocalExecutionPolicyError::InvalidBound {
                    field: "swap_limit_bytes",
                    detail: format!("must be <= {MAX_MEMORY_LIMIT_BYTES}"),
                });
            }
        }
    }
    debug_assert!(resources.memory_limit_bytes != Some(0));
    debug_assert!(
        matches!(resources.memory_mode, LocalEnforcementMode::Disabled) || resources.memory_limit_bytes.is_some()
    );
    Ok(())
}

fn validate_pids_limit(resources: &LocalResourceBudgetPolicy) -> Result<(), LocalExecutionPolicyError> {
    match resources.pids_mode {
        LocalEnforcementMode::Disabled => {}
        LocalEnforcementMode::BestEffort | LocalEnforcementMode::Required => {
            if resources.pids_limit_count.is_none() {
                return Err(LocalExecutionPolicyError::InvalidResourceConfig {
                    field: "pids_limit_count",
                    detail: "must be present when pids enforcement is enabled".to_string(),
                });
            }
        }
    }
    match resources.pids_limit_count {
        None => {}
        Some(limit_count) => {
            if limit_count < MIN_PIDS_LIMIT_COUNT {
                return Err(LocalExecutionPolicyError::InvalidBound {
                    field: "pids_limit_count",
                    detail: format!("must be >= {MIN_PIDS_LIMIT_COUNT}"),
                });
            }
            if limit_count > MAX_PIDS_LIMIT_COUNT {
                return Err(LocalExecutionPolicyError::InvalidBound {
                    field: "pids_limit_count",
                    detail: format!("must be <= {MAX_PIDS_LIMIT_COUNT}"),
                });
            }
        }
    }
    debug_assert!(resources.pids_limit_count != Some(0));
    debug_assert!(
        matches!(resources.pids_mode, LocalEnforcementMode::Disabled) || resources.pids_limit_count.is_some()
    );
    Ok(())
}

fn validate_scratch_limit(resources: &LocalResourceBudgetPolicy) -> Result<(), LocalExecutionPolicyError> {
    match resources.scratch_mode {
        LocalEnforcementMode::Disabled => {}
        LocalEnforcementMode::BestEffort | LocalEnforcementMode::Required => {
            if resources.scratch_limit_bytes.is_none() {
                return Err(LocalExecutionPolicyError::InvalidResourceConfig {
                    field: "scratch_limit_bytes",
                    detail: "must be present when scratch enforcement is enabled".to_string(),
                });
            }
        }
    }
    match resources.scratch_limit_bytes {
        None => {}
        Some(limit_bytes) => {
            if limit_bytes < MIN_SCRATCH_LIMIT_BYTES {
                return Err(LocalExecutionPolicyError::InvalidBound {
                    field: "scratch_limit_bytes",
                    detail: format!("must be >= {MIN_SCRATCH_LIMIT_BYTES}"),
                });
            }
            if limit_bytes > MAX_SCRATCH_LIMIT_BYTES {
                return Err(LocalExecutionPolicyError::InvalidBound {
                    field: "scratch_limit_bytes",
                    detail: format!("must be <= {MAX_SCRATCH_LIMIT_BYTES}"),
                });
            }
        }
    }
    debug_assert!(resources.scratch_limit_bytes != Some(0));
    debug_assert!(
        matches!(resources.scratch_mode, LocalEnforcementMode::Disabled) || resources.scratch_limit_bytes.is_some()
    );
    Ok(())
}

fn convert_wall_deadline(
    wall: LocalWallDeadlinePolicy,
) -> Result<snix_build::buildservice::SandboxWallDeadline, LocalExecutionPolicyError> {
    match wall {
        LocalWallDeadlinePolicy::Bounded { wall_deadline_secs } => {
            debug_assert!(wall_deadline_secs >= MIN_WALL_DEADLINE_SECS);
            debug_assert!(wall_deadline_secs <= MAX_WALL_DEADLINE_SECS);
            let wall_deadline_ms =
                wall_deadline_secs.checked_mul(MS_PER_SEC).ok_or_else(|| LocalExecutionPolicyError::InvalidBound {
                    field: "wall_deadline_secs",
                    detail: "millisecond conversion overflow".to_string(),
                })?;
            Ok(snix_build::buildservice::SandboxWallDeadline::Bounded { wall_deadline_ms })
        }
        LocalWallDeadlinePolicy::Unlimited => Ok(snix_build::buildservice::SandboxWallDeadline::Unlimited),
    }
}

fn convert_silence_budget(
    silence: LocalSilenceBudgetPolicy,
) -> Result<snix_build::buildservice::SandboxSilenceBudget, LocalExecutionPolicyError> {
    match silence {
        LocalSilenceBudgetPolicy::Disabled => Ok(snix_build::buildservice::SandboxSilenceBudget::Disabled),
        LocalSilenceBudgetPolicy::Bounded { silence_timeout_secs } => {
            debug_assert!(silence_timeout_secs >= MIN_SILENCE_TIMEOUT_SECS);
            debug_assert!(silence_timeout_secs <= MAX_SILENCE_TIMEOUT_SECS);
            let silence_timeout_ms = silence_timeout_secs.checked_mul(MS_PER_SEC).ok_or_else(|| {
                LocalExecutionPolicyError::InvalidBound {
                    field: "silence_timeout_secs",
                    detail: "millisecond conversion overflow".to_string(),
                }
            })?;
            Ok(snix_build::buildservice::SandboxSilenceBudget::Bounded { silence_timeout_ms })
        }
    }
}

fn convert_enforcement_mode(mode: LocalEnforcementMode) -> snix_build::buildservice::SandboxEnforcementMode {
    match mode {
        LocalEnforcementMode::Disabled => snix_build::buildservice::SandboxEnforcementMode::Disabled,
        LocalEnforcementMode::BestEffort => snix_build::buildservice::SandboxEnforcementMode::BestEffort,
        LocalEnforcementMode::Required => snix_build::buildservice::SandboxEnforcementMode::Required,
    }
}

fn convert_swap_policy(swap: LocalSwapLimitPolicy) -> snix_build::buildservice::SandboxSwapPolicy {
    match swap {
        LocalSwapLimitPolicy::HostDefault => snix_build::buildservice::SandboxSwapPolicy::HostDefault,
        LocalSwapLimitPolicy::Zero => snix_build::buildservice::SandboxSwapPolicy::Zero,
        LocalSwapLimitPolicy::Bounded { swap_limit_bytes } => {
            snix_build::buildservice::SandboxSwapPolicy::BoundedBytes { swap_limit_bytes }
        }
    }
}

fn convert_helper_scope(policy: LocalHelperScopePolicy) -> snix_build::buildservice::SandboxHelperScopePolicy {
    match policy {
        LocalHelperScopePolicy::RequireContained => {
            snix_build::buildservice::SandboxHelperScopePolicy::RequireContained
        }
        LocalHelperScopePolicy::AllowInProcessPassiveDatagram => {
            snix_build::buildservice::SandboxHelperScopePolicy::AllowInProcessPassiveDatagram
        }
    }
}

fn convert_resource_policy(
    resources: &LocalResourceBudgetPolicy,
) -> Result<snix_build::buildservice::SandboxResourcePolicy, LocalExecutionPolicyError> {
    debug_assert!(resources.memory_limit_bytes != Some(0));
    debug_assert!(resources.pids_limit_count != Some(0));
    Ok(snix_build::buildservice::SandboxResourcePolicy {
        cgroup_root: resources.cgroup_root.clone(),
        memory_mode: convert_enforcement_mode(resources.memory_mode),
        memory_limit_bytes: resources.memory_limit_bytes,
        swap_policy: convert_swap_policy(resources.swap_policy),
        pids_mode: convert_enforcement_mode(resources.pids_mode),
        pids_limit_count: resources.pids_limit_count,
        scratch_mode: convert_enforcement_mode(resources.scratch_mode),
        scratch_limit_bytes: resources.scratch_limit_bytes,
        helper_scope_policy: convert_helper_scope(resources.helper_scope_policy),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeoutEvaluationInput {
    pub is_cancelled: bool,
    pub is_shutdown: bool,
    pub is_completed: bool,
    pub elapsed_ms: u64,
    pub last_output_ms: u64,
    pub wall_deadline_ms: Option<u64>,
    pub silence_timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutDisposition {
    Running,
    Completed,
    Cancelled,
    Shutdown,
    WallDeadlineExceeded,
    SilenceBudgetExceeded,
}

/// Pure deterministic race resolution between cancellation, shutdown, wall
/// deadline, silence budget, and process completion.
pub fn evaluate_timeout_race(input: TimeoutEvaluationInput) -> TimeoutDisposition {
    assert!(input.last_output_ms <= input.elapsed_ms, "last_output_ms must not exceed elapsed_ms");
    let silence_elapsed_ms = input.elapsed_ms.saturating_sub(input.last_output_ms);
    debug_assert!(silence_elapsed_ms <= input.elapsed_ms);

    if input.is_cancelled {
        return TimeoutDisposition::Cancelled;
    }
    if input.is_shutdown {
        return TimeoutDisposition::Shutdown;
    }
    match input.wall_deadline_ms {
        Some(wall_ms) if input.elapsed_ms >= wall_ms => return TimeoutDisposition::WallDeadlineExceeded,
        Some(_) | None => {}
    }
    match input.silence_timeout_ms {
        Some(silence_ms) if silence_elapsed_ms >= silence_ms => {
            return TimeoutDisposition::SilenceBudgetExceeded;
        }
        Some(_) | None => {}
    }
    if input.is_completed {
        return TimeoutDisposition::Completed;
    }
    TimeoutDisposition::Running
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_policy_has_documented_wall_deadline_and_disabled_silence() {
        let policy = LocalExecutionPolicy::standard();
        assert!(policy.validate().is_ok());
        assert_eq!(policy.wall_deadline, LocalWallDeadlinePolicy::Bounded {
            wall_deadline_secs: DEFAULT_WALL_DEADLINE_SECS,
        });
        assert_eq!(policy.silence_budget, LocalSilenceBudgetPolicy::Disabled);
        assert_eq!(policy.shutdown_grace_ms, DEFAULT_SHUTDOWN_GRACE_MS);

        let snix = policy.to_snix_execution_policy().unwrap();
        assert_eq!(snix.wall_deadline, snix_build::buildservice::SandboxWallDeadline::Bounded {
            wall_deadline_ms: DEFAULT_WALL_DEADLINE_SECS * 1_000,
        });
        assert_eq!(snix.silence_budget, snix_build::buildservice::SandboxSilenceBudget::Disabled);
    }

    #[test]
    fn unlimited_policy_is_explicit_opt_out_and_zero_bounds_are_rejected() {
        let unlimited = LocalExecutionPolicy::unlimited();
        assert!(unlimited.validate().is_ok());
        assert_eq!(unlimited.wall_deadline, LocalWallDeadlinePolicy::Unlimited);

        let mut zero_wall = LocalExecutionPolicy::standard();
        zero_wall.wall_deadline = LocalWallDeadlinePolicy::Bounded { wall_deadline_secs: 0 };
        assert!(matches!(
            zero_wall.validate(),
            Err(LocalExecutionPolicyError::InvalidBound {
                field: "wall_deadline_secs",
                ..
            })
        ));

        let mut zero_silence = LocalExecutionPolicy::standard();
        zero_silence.silence_budget = LocalSilenceBudgetPolicy::Bounded {
            silence_timeout_secs: 0,
        };
        assert!(matches!(
            zero_silence.validate(),
            Err(LocalExecutionPolicyError::InvalidBound {
                field: "silence_timeout_secs",
                ..
            })
        ));

        let mut missing_memory = LocalExecutionPolicy::standard();
        missing_memory.resources.memory_mode = LocalEnforcementMode::Required;
        assert!(matches!(
            missing_memory.validate(),
            Err(LocalExecutionPolicyError::InvalidResourceConfig {
                field: "memory_limit_bytes",
                ..
            })
        ));

        let mut tiny_pids = LocalExecutionPolicy::standard();
        tiny_pids.resources.pids_mode = LocalEnforcementMode::Required;
        tiny_pids.resources.pids_limit_count = Some(2);
        assert!(matches!(
            tiny_pids.validate(),
            Err(LocalExecutionPolicyError::InvalidBound {
                field: "pids_limit_count",
                ..
            })
        ));
    }

    #[test]
    fn timeout_race_resolution_is_deterministic_without_clocks() {
        let base = TimeoutEvaluationInput {
            is_cancelled: false,
            is_shutdown: false,
            is_completed: false,
            elapsed_ms: 500,
            last_output_ms: 500,
            wall_deadline_ms: Some(1_000),
            silence_timeout_ms: Some(400),
        };
        assert_eq!(evaluate_timeout_race(base), TimeoutDisposition::Running);

        let completed_before_limits = TimeoutEvaluationInput {
            is_completed: true,
            ..base
        };
        assert_eq!(evaluate_timeout_race(completed_before_limits), TimeoutDisposition::Completed);

        let silence_and_completed = TimeoutEvaluationInput {
            is_completed: true,
            elapsed_ms: 900,
            last_output_ms: 500,
            ..base
        };
        assert_eq!(evaluate_timeout_race(silence_and_completed), TimeoutDisposition::SilenceBudgetExceeded);

        let wall_beats_silence_and_completion = TimeoutEvaluationInput {
            is_completed: true,
            elapsed_ms: 1_000,
            last_output_ms: 500,
            ..base
        };
        assert_eq!(evaluate_timeout_race(wall_beats_silence_and_completion), TimeoutDisposition::WallDeadlineExceeded);

        let cancel_beats_all = TimeoutEvaluationInput {
            is_cancelled: true,
            is_shutdown: true,
            ..wall_beats_silence_and_completion
        };
        assert_eq!(evaluate_timeout_race(cancel_beats_all), TimeoutDisposition::Cancelled);

        let shutdown_beats_wall = TimeoutEvaluationInput {
            is_shutdown: true,
            ..wall_beats_silence_and_completion
        };
        assert_eq!(evaluate_timeout_race(shutdown_beats_wall), TimeoutDisposition::Shutdown);
    }

    #[test]
    fn disabled_resource_modes_reject_zero_and_overflow_bounds_without_panicking() {
        let assert_invalid_bound = |mutate: fn(&mut LocalResourceBudgetPolicy), expected_field: &'static str| {
            let mut policy = LocalExecutionPolicy::standard();
            mutate(&mut policy.resources);
            assert!(matches!(
                policy.validate(),
                Err(LocalExecutionPolicyError::InvalidBound { field, .. }) if field == expected_field
            ));
            assert!(matches!(
                policy.to_snix_execution_policy(),
                Err(LocalExecutionPolicyError::InvalidBound { field, .. }) if field == expected_field
            ));
        };

        assert_invalid_bound(|r| r.memory_limit_bytes = Some(0), "memory_limit_bytes");
        assert_invalid_bound(|r| r.memory_limit_bytes = Some(u64::MAX), "memory_limit_bytes");
        assert_invalid_bound(|r| r.pids_limit_count = Some(0), "pids_limit_count");
        assert_invalid_bound(|r| r.pids_limit_count = Some(u32::MAX), "pids_limit_count");
        assert_invalid_bound(|r| r.scratch_limit_bytes = Some(0), "scratch_limit_bytes");
        assert_invalid_bound(|r| r.scratch_limit_bytes = Some(u64::MAX), "scratch_limit_bytes");

        let mut valid_disabled = LocalExecutionPolicy::standard();
        valid_disabled.resources.memory_limit_bytes = Some(MIN_MEMORY_LIMIT_BYTES);
        valid_disabled.resources.pids_limit_count = Some(MIN_PIDS_LIMIT_COUNT);
        valid_disabled.resources.scratch_limit_bytes = Some(MIN_SCRATCH_LIMIT_BYTES);
        assert!(valid_disabled.validate().is_ok());
        assert!(valid_disabled.to_snix_execution_policy().is_ok());
    }
}
