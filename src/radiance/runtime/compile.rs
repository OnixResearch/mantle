const OUTPUT_BYTES_MAX: usize = 1_048_576;
const SOURCE_DATE_EPOCH: &str = "1";

pub(super) struct Input<'a> {
    pub cc: &'a std::path::Path,
    pub cc_driver: &'a std::path::Path,
    pub source_root: &'a std::path::Path,
    pub profile: &'a crate::radiance::profile::NativeBuild,
    pub source: &'a str,
    pub object: &'a std::path::Path,
}

pub(super) fn run(
    input: Input<'_>,
) -> Result<Vec<crate::protected_exec::ProtectedSeccompAuditEvent>, crate::errors::RunError> {
    let mut command = std::process::Command::new(input.cc);
    command
        .current_dir(input.source_root)
        .env_clear()
        .env("PATH", "")
        .env("LC_ALL", "C")
        .env("NIX_CC_USE_RESPONSE_FILE", "0")
        .env("SOURCE_DATE_EPOCH", SOURCE_DATE_EPOCH)
        .args(&input.profile.flags)
        .arg("-c")
        .arg(input.source)
        .arg("-o")
        .arg(input.object);
    super::network::install_pre_exec(&mut command).map_err(|error| {
        crate::errors::RunError::Internal(format!("installing Radiance compile no-network filter: {error}"))
    })?;
    let policy = policy(input.cc, input.cc_driver, input.profile)?;
    let supervisor = crate::protected_exec_ptrace::install_exec_supervisor(policy).map_err(|error| {
        crate::errors::RunError::Internal(format!("installing Radiance compile ptrace supervisor: {error}"))
    })?;
    let output = supervisor.output(&mut command).map_err(|error| {
        crate::errors::RunError::Internal(format!(
            "launching protected Radiance compile {}: {error}",
            input.cc.display()
        ))
    })?;
    supervisor
        .wait_for_audit_quiescence()
        .map_err(|error| crate::errors::RunError::Internal(format!("waiting for Radiance compile audit: {error}")))?;
    let events = supervisor.audit_events();
    let stage_label = format!("{} compile of {}", input.profile.role, input.source);
    super::native::validate_root_output(&output, &stage_label, OUTPUT_BYTES_MAX, &events)?;
    super::native::validate_audit_shape(
        &events,
        super::native::COMPILE_AUDIT_EVENTS,
        &[input.cc, input.cc_driver],
        &format!("{} compile", input.source),
    )?;
    debug_assert!(input.object.is_file());
    debug_assert_eq!(events.len(), super::native::COMPILE_AUDIT_EVENTS);
    Ok(events)
}

fn policy(
    cc: &std::path::Path,
    cc_driver: &std::path::Path,
    profile: &crate::radiance::profile::NativeBuild,
) -> Result<crate::protected_exec::ProtectedExecPolicy, crate::errors::RunError> {
    let (cc_blake3, _) = crate::radiance::source::digest_file(cc, "Radiance host C compiler launcher")?;
    let (cc_driver_blake3, _) = crate::radiance::source::digest_file(cc_driver, "Radiance host C compiler driver")?;
    let planned = [
        crate::protected_exec::PlannedExecutable {
            authorization_id: format!("{}-cc", profile.role),
            source_stage_id: super::native::PRODUCER_ID.to_string(),
            path: cc.to_path_buf(),
            digest_hex: cc_blake3,
        },
        crate::protected_exec::PlannedExecutable {
            authorization_id: format!("{}-cc-driver", profile.role),
            source_stage_id: super::native::PRODUCER_ID.to_string(),
            path: cc_driver.to_path_buf(),
            digest_hex: cc_driver_blake3,
        },
    ];
    let policy = crate::protected_exec::ProtectedExecPolicy::from_action_plan(
        &[super::native::PRODUCER_ID.to_string()],
        &planned,
    )
    .map_err(|error| crate::errors::RunError::Internal(format!("constructing Radiance compile policy: {error:?}")))?;
    debug_assert_eq!(planned.len(), super::native::COMPILE_AUDIT_EVENTS);
    debug_assert_ne!(planned[0].path, planned[1].path);
    Ok(policy)
}
