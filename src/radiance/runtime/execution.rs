const STAGE_OUTPUT_BYTES_MAX: usize = 8_388_608;
const PLANNED_EXECUTABLE_COUNT: usize = 1;
const EXECUTION_PRODUCER_ID: &str = "radiance-reference-execution";
const POLICY_DECISION_ALLOWED: &str = "allowed";
const INPUT_MODULE_DEFAULT: &str = "compiler/radiance.rad";

pub(in crate::radiance) struct RunRequest<'a> {
    pub plan: &'a crunch_radiance_reference_core::RadianceReferencePlan,
    pub profile: &'a crate::radiance::profile::Definition,
    pub radiance_source: &'a std::path::Path,
    pub seed: &'a std::path::Path,
    pub native_tools: &'a super::native::BuildOutputs,
    pub stages_dir: &'a std::path::Path,
}

pub(in crate::radiance) struct RunOutputs {
    pub stages: Vec<crunch_radiance_reference_core::RadianceStageObservation>,
    pub raw_audit_events: Vec<crate::protected_exec::ProtectedSeccompAuditEvent>,
    pub seed_final: std::path::PathBuf,
    pub c99_final: std::path::PathBuf,
}

pub(in crate::radiance) fn run_stages(request: RunRequest<'_>) -> Result<RunOutputs, crate::errors::RunError> {
    if !request.radiance_source.is_dir() || !request.seed.is_file() {
        return Err(crate::errors::RunError::Internal("Radiance admitted source or seed is missing".to_string()));
    }
    std::fs::create_dir_all(request.stages_dir).map_err(|error| {
        crate::errors::RunError::Internal(format!(
            "creating Radiance stages directory {}: {error}",
            request.stages_dir.display()
        ))
    })?;
    let compiler_arguments = super::arguments::expand(request.radiance_source, request.profile)?;
    let mut observations = Vec::with_capacity(crunch_radiance_reference_core::RADIANCE_STAGE_COUNT);
    let mut raw_audit_events = Vec::with_capacity(crunch_radiance_reference_core::RADIANCE_STAGE_COUNT);
    let mut seed_predecessor = request.seed.to_path_buf();
    let mut c99_predecessor = request.native_tools.bootstrap_compiler.clone();
    for stage in &request.plan.stages {
        let predecessor = match stage.route {
            crunch_radiance_reference_core::RadianceRoute::Seed => &seed_predecessor,
            crunch_radiance_reference_core::RadianceRoute::C99 => &c99_predecessor,
        };
        let launcher = launcher_for_stage(stage, request.native_tools);
        let output = stage_output_path(request.stages_dir, stage);
        let stage_result = execute_stage(StageRequest {
            stage,
            launcher,
            predecessor,
            output: &output,
            radiance_source: request.radiance_source,
            emulator_arguments: &request.profile.emulator_arguments,
            compiler_arguments: &compiler_arguments,
        })?;
        observations.push(stage_result.observation);
        raw_audit_events.extend(stage_result.audit_events);
        match stage.route {
            crunch_radiance_reference_core::RadianceRoute::Seed => seed_predecessor = output,
            crunch_radiance_reference_core::RadianceRoute::C99 => c99_predecessor = output,
        }
    }
    if observations.len() != crunch_radiance_reference_core::RADIANCE_STAGE_COUNT {
        return Err(crate::errors::RunError::Internal("Radiance stage execution count drifted".to_string()));
    }
    debug_assert_eq!(observations.len(), request.plan.stages.len());
    debug_assert_eq!(raw_audit_events.len(), observations.len());
    Ok(RunOutputs {
        stages: observations,
        raw_audit_events,
        seed_final: seed_predecessor,
        c99_final: c99_predecessor,
    })
}

struct StageRequest<'a> {
    stage: &'a crunch_radiance_reference_core::RadianceStagePlan,
    launcher: &'a std::path::Path,
    predecessor: &'a std::path::Path,
    output: &'a std::path::Path,
    radiance_source: &'a std::path::Path,
    emulator_arguments: &'a [String],
    compiler_arguments: &'a [String],
}

struct StageResult {
    observation: crunch_radiance_reference_core::RadianceStageObservation,
    audit_events: Vec<crate::protected_exec::ProtectedSeccompAuditEvent>,
}

struct ObservationInput<'a> {
    request: &'a StageRequest<'a>,
    launcher: crunch_radiance_reference_core::RadianceArtifactObservation,
    predecessor: crunch_radiance_reference_core::RadianceArtifactObservation,
    process_output: &'a std::process::Output,
    audit_events: &'a [crate::protected_exec::ProtectedSeccompAuditEvent],
}

fn execute_stage(request: StageRequest<'_>) -> Result<StageResult, crate::errors::RunError> {
    if request.output.exists() || !request.launcher.is_file() || !request.predecessor.is_file() {
        return Err(crate::errors::RunError::Internal(format!(
            "Radiance stage {} inputs or destination are invalid",
            request.stage.stage
        )));
    }
    let launcher = observe_path(request.launcher, launcher_role(request.stage.launch_kind), "Radiance stage launcher")?;
    let predecessor = observe_path(
        request.predecessor,
        crunch_radiance_reference_core::RadianceArtifactRole::Seed,
        "Radiance stage predecessor",
    )?;
    let policy = crate::protected_exec::ProtectedExecPolicy::from_action_plan(&[EXECUTION_PRODUCER_ID.to_string()], &[
        crate::protected_exec::PlannedExecutable {
            authorization_id: request.stage.output_role.clone(),
            source_stage_id: EXECUTION_PRODUCER_ID.to_string(),
            path: request.launcher.to_path_buf(),
            digest_hex: launcher.digest_blake3.clone(),
        },
    ])
    .map_err(|error| {
        crate::errors::RunError::Internal(format!("constructing Radiance protected-exec policy: {error:?}"))
    })?;
    let supervisor = crate::protected_exec_ptrace::install_exec_supervisor(policy).map_err(|error| {
        crate::errors::RunError::Internal(format!("installing Radiance ptrace supervisor: {error}"))
    })?;
    let mut command = stage_command(&request)?;
    super::network::install_pre_exec(&mut command).map_err(|error| {
        crate::errors::RunError::Internal(format!("installing Radiance no-network filter: {error}"))
    })?;
    let output = supervisor
        .output(&mut command)
        .map_err(|error| crate::errors::RunError::Internal(format!("running protected Radiance stage: {error}")))?;
    supervisor
        .wait_for_audit_quiescence()
        .map_err(|error| crate::errors::RunError::Internal(format!("waiting for Radiance protected audit: {error}")))?;
    let audit_events = supervisor.audit_events();
    validate_stage_process_output(&output, request.stage, &audit_events)?;
    let observation = build_observation(ObservationInput {
        request: &request,
        launcher,
        predecessor,
        process_output: &output,
        audit_events: &audit_events,
    })?;
    write_stage_log(request.output, "stdout", &output.stdout)?;
    write_stage_log(request.output, "stderr", &output.stderr)?;
    debug_assert_eq!(observation.exit_code, 0);
    debug_assert_eq!(observation.denied_event_count, 0);
    Ok(StageResult {
        observation,
        audit_events,
    })
}

fn build_observation(
    input: ObservationInput<'_>,
) -> Result<crunch_radiance_reference_core::RadianceStageObservation, crate::errors::RunError> {
    let exit_code =
        input.process_output.status.code().ok_or_else(|| {
            crate::errors::RunError::Internal("successful Radiance stage has no exit code".to_string())
        })?;
    let produced = observe_path(
        input.request.output,
        crunch_radiance_reference_core::RadianceArtifactRole::Seed,
        "Radiance stage output",
    )?;
    let audit_bytes =
        crate::radiance::source::canonical_pretty_json(&input.audit_events, "Radiance stage protected audit")?;
    let audit_blake3 = blake3::hash(&audit_bytes).to_hex().to_string();
    let event_count = u32::try_from(input.audit_events.len())
        .map_err(|_| crate::errors::RunError::Internal("Radiance protected audit event count overflow".to_string()))?;
    let denied_event_count = u32::try_from(
        input.audit_events.iter().filter(|event| event.policy_decision != POLICY_DECISION_ALLOWED).count(),
    )
    .map_err(|_| crate::errors::RunError::Internal("Radiance denied audit event count overflow".to_string()))?;
    let observation = crunch_radiance_reference_core::RadianceStageObservation {
        route: input.request.stage.route,
        stage: input.request.stage.stage,
        launch_kind: input.request.stage.launch_kind,
        launcher_blake3: input.launcher.digest_blake3,
        predecessor_blake3: input.predecessor.digest_blake3,
        output_role: input.request.stage.output_role.clone(),
        output_blake3: produced.digest_blake3,
        output_bytes: produced.byte_count,
        exit_code,
        stdout_blake3: blake3::hash(&input.process_output.stdout).to_hex().to_string(),
        stderr_blake3: blake3::hash(&input.process_output.stderr).to_hex().to_string(),
        protected_audit_blake3: audit_blake3,
        protected_audit_event_count: event_count,
        denied_event_count,
    };
    debug_assert_eq!(observation.denied_event_count, 0);
    debug_assert_eq!(observation.exit_code, 0);
    Ok(observation)
}

fn stage_command(request: &StageRequest<'_>) -> Result<std::process::Command, crate::errors::RunError> {
    let mut command = std::process::Command::new(request.launcher);
    command.current_dir(request.radiance_source).env_clear().env("LC_ALL", "C");
    if request.stage.launch_kind == crunch_radiance_reference_core::RadianceLaunchKind::EmulatedCompiler {
        command.args(request.emulator_arguments).arg(request.predecessor);
    } else {
        let input_module = INPUT_MODULE_DEFAULT.to_string();
        super::arguments::validate_module(&input_module)?;
        command.arg(input_module);
    }
    command.args(request.compiler_arguments).arg("-o").arg(request.output);
    debug_assert!(!request.compiler_arguments.is_empty());
    Ok(command)
}

fn validate_stage_process_output(
    output: &std::process::Output,
    stage: &crunch_radiance_reference_core::RadianceStagePlan,
    audit_events: &[crate::protected_exec::ProtectedSeccompAuditEvent],
) -> Result<(), crate::errors::RunError> {
    if output.stdout.len() > STAGE_OUTPUT_BYTES_MAX || output.stderr.len() > STAGE_OUTPUT_BYTES_MAX {
        return Err(crate::errors::RunError::Internal(format!(
            "Radiance {} stage {} output exceeded its bound",
            crunch_radiance_reference_core::route_label(stage.route),
            stage.stage
        )));
    }
    if !output.status.success() {
        return Err(crate::errors::RunError::Internal(format!(
            "Radiance {} stage {} failed: {}",
            crunch_radiance_reference_core::route_label(stage.route),
            stage.stage,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    if audit_events.len() != PLANNED_EXECUTABLE_COUNT
        || audit_events.iter().any(|event| event.policy_decision != POLICY_DECISION_ALLOWED)
    {
        return Err(crate::errors::RunError::Internal(format!(
            "Radiance {} stage {} protected audit did not contain one allowed executable",
            crunch_radiance_reference_core::route_label(stage.route),
            stage.stage
        )));
    }
    debug_assert!(output.status.success());
    debug_assert_eq!(audit_events.len(), PLANNED_EXECUTABLE_COUNT);
    Ok(())
}

fn launcher_for_stage<'a>(
    stage: &crunch_radiance_reference_core::RadianceStagePlan,
    tools: &'a super::native::BuildOutputs,
) -> &'a std::path::Path {
    match stage.launch_kind {
        crunch_radiance_reference_core::RadianceLaunchKind::NativeBootstrap => &tools.bootstrap_compiler,
        crunch_radiance_reference_core::RadianceLaunchKind::EmulatedCompiler => &tools.emulator,
    }
}

const fn launcher_role(
    kind: crunch_radiance_reference_core::RadianceLaunchKind,
) -> crunch_radiance_reference_core::RadianceArtifactRole {
    match kind {
        crunch_radiance_reference_core::RadianceLaunchKind::NativeBootstrap => {
            crunch_radiance_reference_core::RadianceArtifactRole::BootstrapCompiler
        }
        crunch_radiance_reference_core::RadianceLaunchKind::EmulatedCompiler => {
            crunch_radiance_reference_core::RadianceArtifactRole::Emulator
        }
    }
}

fn observe_path(
    path: &std::path::Path,
    role: crunch_radiance_reference_core::RadianceArtifactRole,
    label: &str,
) -> Result<crunch_radiance_reference_core::RadianceArtifactObservation, crate::errors::RunError> {
    let (digest_blake3, byte_count) = crate::radiance::source::digest_file(path, label)?;
    Ok(crunch_radiance_reference_core::RadianceArtifactObservation {
        role,
        digest_blake3,
        byte_count,
    })
}

fn stage_output_path(
    stages_dir: &std::path::Path,
    stage: &crunch_radiance_reference_core::RadianceStagePlan,
) -> std::path::PathBuf {
    stages_dir.join(format!("{}-stage-{}.rv64", crunch_radiance_reference_core::route_label(stage.route), stage.stage))
}

fn write_stage_log(output_path: &std::path::Path, stream: &str, bytes: &[u8]) -> Result<(), crate::errors::RunError> {
    let file_name = output_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| crate::errors::RunError::Internal("Radiance stage output name is not UTF-8".to_string()))?;
    let path = output_path.with_file_name(format!("{file_name}.{stream}"));
    std::fs::write(&path, bytes).map_err(|error| {
        crate::errors::RunError::Internal(format!("writing Radiance stage {stream} log {}: {error}", path.display()))
    })?;
    debug_assert!(path.is_file());
    debug_assert!(bytes.len() <= STAGE_OUTPUT_BYTES_MAX);
    Ok(())
}
