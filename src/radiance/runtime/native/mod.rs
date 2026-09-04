mod authority;
mod tool;

pub(super) const COMPILE_AUDIT_EVENTS: usize = 2;
pub(super) const LINK_AUDIT_EVENTS: usize = 1;
pub(super) const PRODUCER_ID: &str = "radiance-reference-native-build";
pub(super) const DYNAMIC_LINKER_NAME: &str = "ld-linux-x86-64.so.2";
pub(super) const CRT_BEGIN: &str = "crtbeginS.o";
pub(super) const CRT_END: &str = "crtendS.o";
pub(super) const CRT_START_PIE: &str = "Scrt1.o";
pub(super) const CRT_INIT: &str = "crti.o";
pub(super) const CRT_FINI: &str = "crtn.o";
pub(super) const OBJECT_SUFFIX: &str = ".o";

const POLICY_DECISION_ALLOWED: &str = "allowed";

pub(in crate::radiance) struct BuildRequest<'a> {
    pub cc: &'a std::path::Path,
    pub cc_driver: &'a std::path::Path,
    pub linker: &'a std::path::Path,
    pub crt_dir: &'a std::path::Path,
    pub libgcc_dir: &'a std::path::Path,
    pub bootstrap_source: &'a std::path::Path,
    pub emulator_source: &'a std::path::Path,
    pub tools_dir: &'a std::path::Path,
    pub profile: &'a crate::radiance::profile::Definition,
}

pub(in crate::radiance) struct BuildOutputs {
    pub bootstrap_compiler: std::path::PathBuf,
    pub emulator: std::path::PathBuf,
    pub tool_observations: Vec<crunch_radiance_reference_core::RadianceArtifactObservation>,
    pub protected_audit_events: Vec<crate::protected_exec::ProtectedSeccompAuditEvent>,
}

pub(in crate::radiance) fn build_tools(input: BuildRequest<'_>) -> Result<BuildOutputs, crate::errors::RunError> {
    crate::radiance::source::require_absolute_executable(input.cc, "Radiance host C compiler launcher")?;
    crate::radiance::source::require_absolute_executable(input.cc_driver, "Radiance host C compiler driver")?;
    crate::radiance::source::require_absolute_executable(input.linker, "Radiance host linker")?;
    authority::require_link_inputs(input.crt_dir, input.libgcc_dir)?;
    let runtime_observations_before = authority::observe_link_inputs(input.crt_dir, input.libgcc_dir)?;
    if !input.tools_dir.is_dir() {
        std::fs::create_dir_all(input.tools_dir).map_err(|error| {
            crate::errors::RunError::Internal(format!(
                "creating Radiance tools directory {}: {error}",
                input.tools_dir.display()
            ))
        })?;
    }
    let bootstrap_profile = tool::select_profile(input.profile, "bootstrap-compiler")?;
    let emulator_profile = tool::select_profile(input.profile, "emulator")?;
    let bootstrap = tool::produce(tool::Request {
        cc: input.cc,
        cc_driver: input.cc_driver,
        linker: input.linker,
        crt_dir: input.crt_dir,
        libgcc_dir: input.libgcc_dir,
        source_root: input.bootstrap_source,
        tools_dir: input.tools_dir,
        profile: bootstrap_profile,
    })?;
    let emulator = tool::produce(tool::Request {
        cc: input.cc,
        cc_driver: input.cc_driver,
        linker: input.linker,
        crt_dir: input.crt_dir,
        libgcc_dir: input.libgcc_dir,
        source_root: input.emulator_source,
        tools_dir: input.tools_dir,
        profile: emulator_profile,
    })?;
    debug_assert!(input.cc.is_absolute());
    debug_assert!(input.tools_dir.is_dir());
    finish_build(&input, runtime_observations_before, bootstrap, emulator)
}

fn finish_build(
    input: &BuildRequest<'_>,
    runtime_observations_before: [crunch_radiance_reference_core::RadianceArtifactObservation; 2],
    bootstrap: tool::Output,
    emulator: tool::Output,
) -> Result<BuildOutputs, crate::errors::RunError> {
    let runtime_observations_after = authority::observe_link_inputs(input.crt_dir, input.libgcc_dir)?;
    if runtime_observations_after != runtime_observations_before {
        return Err(crate::errors::RunError::Internal(
            "Radiance link runtime inputs changed during the protected native build".to_string(),
        ));
    }
    let observations = vec![
        authority::observe_artifact(
            input.cc,
            crunch_radiance_reference_core::RadianceArtifactRole::HostCompilerLauncher,
            "host C compiler launcher",
        )?,
        authority::observe_artifact(
            input.cc_driver,
            crunch_radiance_reference_core::RadianceArtifactRole::HostCompilerDriver,
            "host C compiler driver",
        )?,
        authority::observe_artifact(
            input.linker,
            crunch_radiance_reference_core::RadianceArtifactRole::HostLinker,
            "host linker",
        )?,
        runtime_observations_before[0].clone(),
        runtime_observations_before[1].clone(),
        authority::observe_artifact(
            &bootstrap.path,
            crunch_radiance_reference_core::RadianceArtifactRole::BootstrapCompiler,
            "built bootstrap compiler",
        )?,
        authority::observe_artifact(
            &emulator.path,
            crunch_radiance_reference_core::RadianceArtifactRole::Emulator,
            "built emulator",
        )?,
    ];
    let mut audit_events = bootstrap.audit_events;
    audit_events
        .try_reserve(emulator.audit_events.len())
        .map_err(|error| crate::errors::RunError::Internal(format!("reserving native audit events: {error}")))?;
    audit_events.extend(emulator.audit_events);
    debug_assert_eq!(observations.len(), crunch_radiance_reference_core::RADIANCE_TOOL_COUNT.saturating_sub(1));
    debug_assert!(bootstrap.path.is_file());
    Ok(BuildOutputs {
        bootstrap_compiler: bootstrap.path,
        emulator: emulator.path,
        tool_observations: observations,
        protected_audit_events: audit_events,
    })
}

pub(super) fn validate_root_output(
    output: &std::process::Output,
    stage: &str,
    bytes_max: usize,
    events: &[crate::protected_exec::ProtectedSeccompAuditEvent],
) -> Result<(), crate::errors::RunError> {
    if output.stdout.len() > bytes_max || output.stderr.len() > bytes_max {
        return Err(crate::errors::RunError::Internal(format!("Radiance {stage} output exceeded its bound")));
    }
    if !output.status.success() {
        let audit_summary = events
            .iter()
            .map(|event| format!("{}:{}:{}", event.policy_decision, event.executable_path.display(), event.reason))
            .collect::<Vec<_>>()
            .join(" | ");
        return Err(crate::errors::RunError::Internal(format!(
            "Radiance {stage} failed with status {}: {}; audit: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim(),
            audit_summary
        )));
    }
    debug_assert!(output.status.success());
    debug_assert!(output.stdout.len() <= bytes_max);
    Ok(())
}

pub(super) fn validate_audit_shape(
    events: &[crate::protected_exec::ProtectedSeccompAuditEvent],
    expected_count: usize,
    expected_paths: &[&std::path::Path],
    stage: &str,
) -> Result<(), crate::errors::RunError> {
    if events.len() != expected_count || events.iter().any(|event| event.policy_decision != POLICY_DECISION_ALLOWED) {
        let summary = events
            .iter()
            .map(|event| format!("{}:{}", event.policy_decision, event.executable_path.display()))
            .collect::<Vec<_>>()
            .join(" | ");
        return Err(crate::errors::RunError::Internal(format!(
            "Radiance {stage} protected audit expected {expected_count} allowed events, got: {summary}"
        )));
    }
    let is_every_path_expected = events.iter().zip(expected_paths).all(|(event, path)| event.executable_path == *path);
    if !is_every_path_expected {
        return Err(crate::errors::RunError::Internal(format!("Radiance {stage} protected audit paths drifted")));
    }
    debug_assert_eq!(events.len(), expected_count);
    debug_assert!(events.iter().all(|event| event.policy_decision == POLICY_DECISION_ALLOWED));
    Ok(())
}

pub(in crate::radiance) fn require_link_inputs(
    crt_dir: &std::path::Path,
    libgcc_dir: &std::path::Path,
) -> Result<(), crate::errors::RunError> {
    authority::require_link_inputs(crt_dir, libgcc_dir)
}

pub(in crate::radiance) fn observe_artifact(
    path: &std::path::Path,
    role: crunch_radiance_reference_core::RadianceArtifactRole,
    label: &str,
) -> Result<crunch_radiance_reference_core::RadianceArtifactObservation, crate::errors::RunError> {
    authority::observe_artifact(path, role, label)
}
