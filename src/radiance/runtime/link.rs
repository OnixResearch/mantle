const OUTPUT_BYTES_MAX: usize = 1_048_576;
const EMULATION: &str = "elf_x86_64";
const HASH_STYLE: &str = "gnu";
const THREAD_COUNT: &str = "1";

pub(super) struct Input<'a> {
    pub linker: &'a std::path::Path,
    pub crt_dir: &'a std::path::Path,
    pub libgcc_dir: &'a std::path::Path,
    pub objects: &'a [std::path::PathBuf],
    pub output_path: &'a std::path::Path,
    pub profile: &'a crate::radiance::profile::NativeBuild,
}

pub(super) fn run(
    input: Input<'_>,
) -> Result<Vec<crate::protected_exec::ProtectedSeccompAuditEvent>, crate::errors::RunError> {
    let stack_size_bytes = input
        .profile
        .linker_flags
        .iter()
        .find_map(|flag| flag.strip_prefix("-Wl,-z,stack-size="))
        .ok_or_else(|| {
        crate::errors::RunError::Internal(format!(
            "Radiance {} profile lacks the stack-size linker flag",
            input.profile.role
        ))
    })?;
    let mut command = command(&input, stack_size_bytes);
    super::network::install_pre_exec(&mut command).map_err(|error| {
        crate::errors::RunError::Internal(format!("installing Radiance link no-network filter: {error}"))
    })?;
    let (linker_blake3, _) = crate::radiance::source::digest_file(input.linker, "Radiance host linker")?;
    let policy =
        crate::protected_exec::ProtectedExecPolicy::from_action_plan(&[super::native::PRODUCER_ID.to_string()], &[
            crate::protected_exec::PlannedExecutable {
                authorization_id: format!("{}-linker", input.profile.role),
                source_stage_id: super::native::PRODUCER_ID.to_string(),
                path: input.linker.to_path_buf(),
                digest_hex: linker_blake3,
            },
        ])
        .map_err(|error| crate::errors::RunError::Internal(format!("constructing Radiance link policy: {error:?}")))?;
    let supervisor = crate::protected_exec_ptrace::install_exec_supervisor(policy).map_err(|error| {
        crate::errors::RunError::Internal(format!("installing Radiance link ptrace supervisor: {error}"))
    })?;
    let output = supervisor.output(&mut command).map_err(|error| {
        crate::errors::RunError::Internal(format!(
            "launching protected Radiance link {}: {error}",
            input.linker.display()
        ))
    })?;
    supervisor
        .wait_for_audit_quiescence()
        .map_err(|error| crate::errors::RunError::Internal(format!("waiting for Radiance link audit: {error}")))?;
    let events = supervisor.audit_events();
    super::native::validate_root_output(&output, &format!("{} link", input.profile.role), OUTPUT_BYTES_MAX, &events)?;
    super::native::validate_audit_shape(&events, super::native::LINK_AUDIT_EVENTS, &[input.linker], "link")?;
    debug_assert!(input.output_path.is_file());
    debug_assert_eq!(events.len(), super::native::LINK_AUDIT_EVENTS);
    Ok(events)
}

fn command(input: &Input<'_>, stack_size_bytes: &str) -> std::process::Command {
    let dynamic_linker = input.crt_dir.join(super::native::DYNAMIC_LINKER_NAME);
    let mut command = std::process::Command::new(input.linker);
    command
        .current_dir("/")
        .env_clear()
        .env("LC_ALL", "C")
        .args(["--threads", THREAD_COUNT, "--hash-style", HASH_STYLE, "--eh-frame-hdr"])
        .args(["-m", EMULATION, "-pie", "-o"])
        .arg(input.output_path)
        .arg(input.crt_dir.join(super::native::CRT_START_PIE))
        .arg(input.crt_dir.join(super::native::CRT_INIT))
        .arg(input.libgcc_dir.join(super::native::CRT_BEGIN))
        .arg("-L")
        .arg(input.crt_dir)
        .arg("-L")
        .arg(input.libgcc_dir)
        .arg("-dynamic-linker")
        .arg(&dynamic_linker);
    for object in input.objects {
        command.arg(object);
    }
    command
        .args(["-lgcc", "-lgcc_eh", "-lc", "-lgcc", "-lgcc_eh"])
        .arg(input.libgcc_dir.join(super::native::CRT_END))
        .arg(input.crt_dir.join(super::native::CRT_FINI))
        .arg("-z")
        .arg(format!("stack-size={stack_size_bytes}"));
    debug_assert!(!input.objects.is_empty());
    debug_assert!(!stack_size_bytes.is_empty());
    command
}
