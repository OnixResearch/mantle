use std::process::Command;
use std::process::Stdio;

use crunch_bootstrap_core::SourceRootCapabilityObservations;
use crunch_bootstrap_core::SourceRootCapabilityReport;
use crunch_bootstrap_core::evaluate_source_root_capabilities;
use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::CommandFamily;
use mantle_application_contract::EffectId;
use mantle_application_contract::EffectKind;
use mantle_application_contract::EffectMeasure;
use mantle_application_contract::EffectOutput;
use mantle_application_contract::EffectSpec;
use mantle_application_contract::ExpectedOutput;
use mantle_application_contract::Observation;
use mantle_application_contract::ObservationStatus;
use mantle_application_contract::classify_observations;
use mantle_application_contract::plan_effects;

use crate::errors::RunError;
use crate::source_root_provider::HOST_COMPILER_PATH_READ_MAX;
use crate::source_root_provider::HOST_COMPILER_PROCESS_MAX;
use crate::source_root_provider::HostCompilerProbeObserver;
use crate::source_root_provider::HostCompilerProcessStatus;

const VERSION_ARGUMENT: &str = "--version";
const HOST_PROCESS_EFFECT: &str = "bootstrap-source-root-host-process-probes";
const HOST_ENVIRONMENT_EFFECT: &str = "bootstrap-source-root-host-path-reads";
const HOST_PROCESS_MAX: u32 = HOST_COMPILER_PROCESS_MAX + 2;

trait SourceRootHostPort {
    fn probe(&mut self) -> SourceRootCapabilityObservations;
    fn process_count(&self) -> u32;
    fn environment_read_count(&self) -> u32;
}

#[derive(Default)]
struct LocalSourceRootHost {
    process_count: u32,
    failed_compiler_processes: u32,
    environment_read_count: u32,
}

impl HostCompilerProbeObserver for LocalSourceRootHost {
    fn path(&mut self) -> Option<std::ffi::OsString> {
        let value = std::env::var_os("PATH");
        self.environment_read_count += 1;
        value
    }

    fn process(&mut self, _program: &std::path::Path, status: HostCompilerProcessStatus) {
        self.process_count += 1;
        if !matches!(status, HostCompilerProcessStatus::Exited { success: true, .. }) {
            self.failed_compiler_processes += 1;
        }
    }
}

impl SourceRootHostPort for LocalSourceRootHost {
    fn probe(&mut self) -> SourceRootCapabilityObservations {
        let host_c_compiler_available = crate::source_root_provider::host_compiler_available_for_source_root(self);
        let host_make_available = command_available("make", &mut self.process_count);
        let host_tar_available = command_available("tar", &mut self.process_count);
        debug_assert!(self.failed_compiler_processes <= self.process_count);
        SourceRootCapabilityObservations {
            platform_supported: cfg!(target_os = "linux"),
            host_c_compiler_available,
            host_make_available,
            host_tar_available,
        }
    }

    fn process_count(&self) -> u32 {
        self.process_count
    }

    fn environment_read_count(&self) -> u32 {
        self.environment_read_count
    }
}

// r[impl mantle.build_correctness.source_root_capability]
// r[impl mantle.build_correctness.source_root_capability.boundary]
pub fn cmd_source_root_capabilities(json: bool) -> Result<(), RunError> {
    let plan = plan_effects(CommandFamily::Bootstrap, &[
        EffectSpec {
            effect_id: HOST_PROCESS_EFFECT,
            kind: EffectKind::RunProcess,
            limit: EffectMeasure::Calls(HOST_PROCESS_MAX),
            expected_output: ExpectedOutput::None,
        },
        EffectSpec {
            effect_id: HOST_ENVIRONMENT_EFFECT,
            kind: EffectKind::ReadEnvironment,
            limit: EffectMeasure::Calls(HOST_COMPILER_PATH_READ_MAX),
            expected_output: ExpectedOutput::None,
        },
    ])
    .map_err(|error| RunError::Internal(format!("planning source-root host probes: {}", error.code())))?;
    let mut port = LocalSourceRootHost::default();
    let capability_result = evaluate_source_root_capabilities(port.probe());
    let outcome = classify_observations(&plan, &[
        Observation {
            effect_id: EffectId(HOST_PROCESS_EFFECT.to_string()),
            kind: EffectKind::RunProcess,
            status: ObservationStatus::Succeeded,
            output: EffectOutput::None,
            usage: EffectMeasure::Calls(port.process_count()),
            diagnostics_code: None,
        },
        Observation {
            effect_id: EffectId(HOST_ENVIRONMENT_EFFECT.to_string()),
            kind: EffectKind::ReadEnvironment,
            status: ObservationStatus::Succeeded,
            output: EffectOutput::None,
            usage: EffectMeasure::Calls(port.environment_read_count()),
            diagnostics_code: None,
        },
    ]);
    if outcome != ApplicationOutcome::Completed {
        return Err(RunError::Internal(format!("source-root host probe observations inconsistent: {outcome:?}")));
    }
    debug_assert!(!capability_result.schema.is_empty());
    debug_assert!(!capability_result.operations.is_empty());
    if json {
        let rendered = serde_json::to_string(&capability_result)
            .map_err(|error| RunError::Internal(format!("serializing source-root capability report: {error}")))?;
        println!("{rendered}");
    } else {
        render_human(&capability_result);
    }
    Ok(())
}

fn command_available(command: &str, attempts: &mut u32) -> bool {
    debug_assert!(!VERSION_ARGUMENT.is_empty());
    debug_assert!(VERSION_ARGUMENT.starts_with('-'));
    let status = Command::new(command)
        .arg(VERSION_ARGUMENT)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    *attempts += 1;
    status.is_ok_and(|status| status.success())
}

fn render_human(capability_result: &SourceRootCapabilityReport) {
    debug_assert!(!capability_result.schema.is_empty());
    debug_assert!(!capability_result.operations.is_empty());
    println!("source-root capability schema: {}", capability_result.schema);
    for capability in &capability_result.operations {
        println!("operation: {}", capability.operation);
        println!("  status: {}", capability.status);
        println!("  capability class: {}", capability.capability_class);
        if let Some(command) = &capability.command {
            println!("  command: {}", command.join(" "));
        }
        for blocker in &capability.blockers {
            println!("  blocker: {blocker}");
        }
        for influence in &capability.host_influences {
            println!("  host influence: {influence}");
        }
        for non_claim in &capability.non_claims {
            println!("  non-claim: {non_claim}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_command_probe_is_false() {
        let mut attempts = 0;
        assert!(!command_available("mantle-command-that-must-not-exist", &mut attempts));
        assert!(!command_available("", &mut attempts));
        assert_eq!(attempts, 2);
    }

    #[test]
    fn host_probe_budget_rejects_an_observation_larger_than_real_subprocess_capacity() {
        let mut executed = 0;
        assert!(command_available("tar", &mut executed));
        assert_eq!(executed, 1);
        let plan = plan_effects(CommandFamily::Bootstrap, &[EffectSpec {
            effect_id: HOST_PROCESS_EFFECT,
            kind: EffectKind::RunProcess,
            limit: EffectMeasure::Calls(HOST_PROCESS_MAX),
            expected_output: ExpectedOutput::None,
        }])
        .unwrap();
        let outcome = classify_observations(&plan, &[Observation {
            effect_id: EffectId(HOST_PROCESS_EFFECT.to_string()),
            kind: EffectKind::RunProcess,
            status: ObservationStatus::Succeeded,
            output: EffectOutput::None,
            usage: EffectMeasure::Calls(HOST_PROCESS_MAX + executed),
            diagnostics_code: None,
        }]);
        assert!(matches!(outcome, ApplicationOutcome::Contradicted { effect_count: 1 }));
    }
}
