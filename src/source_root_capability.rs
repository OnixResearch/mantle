use std::process::Command;
use std::process::Stdio;

use crunch_bootstrap_core::SourceRootCapabilityObservations;
use crunch_bootstrap_core::SourceRootCapabilityReport;
use crunch_bootstrap_core::evaluate_source_root_capabilities;

use crate::errors::RunError;

const VERSION_ARGUMENT: &str = "--version";

// r[impl mantle.build_correctness.source_root_capability]
// r[impl mantle.build_correctness.source_root_capability.boundary]
pub fn cmd_source_root_capabilities(json: bool) -> Result<(), RunError> {
    let capability_result = probe_source_root_capabilities();
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

fn probe_source_root_capabilities() -> SourceRootCapabilityReport {
    let observations = SourceRootCapabilityObservations {
        platform_supported: cfg!(target_os = "linux"),
        host_c_compiler_available: crate::source_root_provider::host_compiler_available_for_source_root(),
        host_make_available: command_available("make"),
        host_tar_available: command_available("tar"),
    };
    let capability_result = evaluate_source_root_capabilities(observations);
    debug_assert!(!capability_result.schema.is_empty());
    debug_assert!(!capability_result.operations.is_empty());
    capability_result
}

fn command_available(command: &str) -> bool {
    debug_assert!(!VERSION_ARGUMENT.is_empty());
    debug_assert!(VERSION_ARGUMENT.starts_with('-'));
    Command::new(command)
        .arg(VERSION_ARGUMENT)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
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
        assert!(!command_available("mantle-command-that-must-not-exist"));
        assert!(!command_available(""));
    }
}
