use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const SOURCE_ROOT_CAPABILITY_REPORT_SCHEMA: &str = "mantle-source-root-capability-report-v1";
pub const SOURCE_ROOT_BOOTSTRAP_OPERATION: &str = "bootstrap-source-root-materialization";
pub const SOURCE_ROOT_SELF_BUILD_OPERATION: &str = "self-build-source-root";
pub const SOURCE_ROOT_STATUS_SUPPORTED_HOST_ASSISTED: &str = "supported-host-assisted";
pub const SOURCE_ROOT_STATUS_UNSUPPORTED: &str = "unsupported";
pub const SOURCE_ROOT_CAPABILITY_CLASS: &str = "host-assisted-source-materialization";
pub const SOURCE_ROOT_NON_CLAIM_FULL_SOURCE_BOOTSTRAP: &str =
    "not a full-source bootstrap: host compiler, build tools, and runtime influence the produced provider";
pub const SOURCE_ROOT_SELF_BUILD_UNSUPPORTED_REASON: &str =
    "Mantle self-build has no executable full-source source-root provider chain";
const SOURCE_ROOT_OPERATION_COUNT: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRootCapabilityObservations {
    pub platform_supported: bool,
    pub host_c_compiler_available: bool,
    pub host_make_available: bool,
    pub host_tar_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRootOperationCapability {
    pub operation: String,
    pub status: String,
    pub capability_class: String,
    pub command: Option<Vec<String>>,
    pub blockers: Vec<String>,
    pub host_influences: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRootCapabilityReport {
    pub schema: String,
    pub observations: SourceRootCapabilityObservations,
    pub operations: Vec<SourceRootOperationCapability>,
}

// r[impl mantle.build_correctness.source_root_capability.boundary]
pub fn evaluate_source_root_capabilities(observations: SourceRootCapabilityObservations) -> SourceRootCapabilityReport {
    let bootstrap = evaluate_bootstrap_capability(&observations);
    let self_build = SourceRootOperationCapability {
        operation: SOURCE_ROOT_SELF_BUILD_OPERATION.to_string(),
        status: SOURCE_ROOT_STATUS_UNSUPPORTED.to_string(),
        capability_class: SOURCE_ROOT_CAPABILITY_CLASS.to_string(),
        command: None,
        blockers: vec![SOURCE_ROOT_SELF_BUILD_UNSUPPORTED_REASON.to_string()],
        host_influences: host_influences(),
        non_claims: vec![SOURCE_ROOT_NON_CLAIM_FULL_SOURCE_BOOTSTRAP.to_string()],
    };
    let report = SourceRootCapabilityReport {
        schema: SOURCE_ROOT_CAPABILITY_REPORT_SCHEMA.to_string(),
        observations,
        operations: vec![bootstrap, self_build],
    };
    debug_assert_eq!(report.operations.len(), SOURCE_ROOT_OPERATION_COUNT);
    debug_assert!(report.operations.iter().all(|operation| !operation.status.is_empty()));
    report
}

fn evaluate_bootstrap_capability(observations: &SourceRootCapabilityObservations) -> SourceRootOperationCapability {
    let mut blockers = Vec::new();
    if !observations.platform_supported {
        blockers.push("source-root materialization is supported only on Linux".to_string());
    }
    if !observations.host_c_compiler_available {
        blockers.push("no host C compiler is available".to_string());
    }
    if !observations.host_make_available {
        blockers.push("host make is unavailable".to_string());
    }
    if !observations.host_tar_available {
        blockers.push("host tar is unavailable".to_string());
    }
    let supported = blockers.is_empty();
    let command = supported.then(|| {
        vec![
            "mantle".to_string(),
            "bootstrap".to_string(),
            "--source-root".to_string(),
            "<source-root-manifest.json>".to_string(),
        ]
    });
    SourceRootOperationCapability {
        operation: SOURCE_ROOT_BOOTSTRAP_OPERATION.to_string(),
        status: if supported {
            SOURCE_ROOT_STATUS_SUPPORTED_HOST_ASSISTED
        } else {
            SOURCE_ROOT_STATUS_UNSUPPORTED
        }
        .to_string(),
        capability_class: SOURCE_ROOT_CAPABILITY_CLASS.to_string(),
        command,
        blockers,
        host_influences: host_influences(),
        non_claims: vec![SOURCE_ROOT_NON_CLAIM_FULL_SOURCE_BOOTSTRAP.to_string()],
    }
}

fn host_influences() -> Vec<String> {
    vec![
        "host C compiler and linker".to_string(),
        "host make and archive extraction tools".to_string(),
        "host kernel and runtime libraries during materialization".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn available() -> SourceRootCapabilityObservations {
        SourceRootCapabilityObservations {
            platform_supported: true,
            host_c_compiler_available: true,
            host_make_available: true,
            host_tar_available: true,
        }
    }

    #[test]
    fn bootstrap_is_reported_as_host_assisted_and_self_build_is_unsupported() {
        let report = evaluate_source_root_capabilities(available());
        assert_eq!(report.operations[0].status, SOURCE_ROOT_STATUS_SUPPORTED_HOST_ASSISTED);
        assert!(report.operations[0].command.is_some());
        assert_eq!(report.operations[1].status, SOURCE_ROOT_STATUS_UNSUPPORTED);
        assert!(report.operations[1].command.is_none());
    }

    #[test]
    fn missing_host_compiler_removes_bootstrap_command() {
        let mut observations = available();
        observations.host_c_compiler_available = false;
        let report = evaluate_source_root_capabilities(observations);
        assert_eq!(report.operations[0].status, SOURCE_ROOT_STATUS_UNSUPPORTED);
        assert!(report.operations[0].command.is_none());
        assert!(report.operations[0].blockers.iter().any(|item| item.contains("compiler")));
    }
}
