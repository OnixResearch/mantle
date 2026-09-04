pub(super) struct Output {
    pub path: std::path::PathBuf,
    pub audit_events: Vec<crate::protected_exec::ProtectedSeccompAuditEvent>,
}

pub(super) struct Request<'a> {
    pub cc: &'a std::path::Path,
    pub cc_driver: &'a std::path::Path,
    pub linker: &'a std::path::Path,
    pub crt_dir: &'a std::path::Path,
    pub libgcc_dir: &'a std::path::Path,
    pub source_root: &'a std::path::Path,
    pub tools_dir: &'a std::path::Path,
    pub profile: &'a crate::radiance::profile::NativeBuild,
}

pub(super) fn select_profile<'a>(
    profile: &'a crate::radiance::profile::Definition,
    role: &str,
) -> Result<&'a crate::radiance::profile::NativeBuild, crate::errors::RunError> {
    let selected = match role {
        "bootstrap-compiler" => &profile.native_builds.bootstrap_compiler,
        "emulator" => &profile.native_builds.emulator,
        other => {
            return Err(crate::errors::RunError::Internal(format!("unsupported Radiance native profile role {other}")));
        }
    };
    debug_assert_eq!(selected.role, role);
    debug_assert!(!selected.sources.is_empty());
    Ok(selected)
}

pub(super) fn produce(input: Request<'_>) -> Result<Output, crate::errors::RunError> {
    if !input.source_root.is_dir() {
        return Err(crate::errors::RunError::Internal(format!(
            "Radiance native source root is missing: {}",
            input.source_root.display()
        )));
    }
    let output_path = input.tools_dir.join(&input.profile.output_name);
    if output_path.exists() {
        return Err(crate::errors::RunError::Internal(format!(
            "Radiance native output already exists: {}",
            output_path.display()
        )));
    }
    let object_dir = input.tools_dir.join(format!("{}.objects", input.profile.output_name));
    std::fs::create_dir(&object_dir).map_err(|error| {
        crate::errors::RunError::Internal(format!(
            "creating Radiance object directory {}: {error}",
            object_dir.display()
        ))
    })?;
    let compile_event_count_max =
        input.profile.sources.len().checked_mul(super::COMPILE_AUDIT_EVENTS).ok_or_else(|| {
            crate::errors::RunError::Internal("Radiance compile audit event count overflow".to_string())
        })?;
    let audit_event_count_max = compile_event_count_max
        .checked_add(super::LINK_AUDIT_EVENTS)
        .ok_or_else(|| crate::errors::RunError::Internal("Radiance native audit event count overflow".to_string()))?;
    let mut audit_events = Vec::with_capacity(audit_event_count_max);
    let mut objects = Vec::with_capacity(input.profile.sources.len());
    for (index, source) in input.profile.sources.iter().enumerate() {
        super::authority::validate_source_argument(source)?;
        let object = object_dir.join(format!("{index:03}{}", super::OBJECT_SUFFIX));
        audit_events.extend(crate::radiance::runtime::compile::run(crate::radiance::runtime::compile::Input {
            cc: input.cc,
            cc_driver: input.cc_driver,
            source_root: input.source_root,
            profile: input.profile,
            source,
            object: &object,
        })?);
        objects.push(object);
    }
    audit_events.extend(crate::radiance::runtime::link::run(crate::radiance::runtime::link::Input {
        linker: input.linker,
        crt_dir: input.crt_dir,
        libgcc_dir: input.libgcc_dir,
        objects: &objects,
        output_path: &output_path,
        profile: input.profile,
    })?);
    if !output_path.is_file() {
        return Err(crate::errors::RunError::Internal(format!(
            "Radiance native build omitted {}",
            output_path.display()
        )));
    }
    debug_assert!(output_path.starts_with(input.tools_dir));
    debug_assert_eq!(audit_events.len(), audit_event_count_max);
    Ok(Output {
        path: output_path,
        audit_events,
    })
}
