const CRT_INPUTS_PAYLOAD_ID: &str = "radiance-host-crt-inputs";
const LIBGCC_INPUTS_PAYLOAD_ID: &str = "radiance-host-libgcc-inputs";
const CRT_REQUIRED_FILES: &[&str] = &[
    super::CRT_START_PIE,
    super::CRT_INIT,
    super::CRT_FINI,
    super::DYNAMIC_LINKER_NAME,
    "libc.so",
];
const LIBGCC_REQUIRED_FILES: &[&str] = &[super::CRT_BEGIN, super::CRT_END, "libgcc.a", "libgcc_eh.a"];

pub(super) fn observe_link_inputs(
    crt_dir: &std::path::Path,
    libgcc_dir: &std::path::Path,
) -> Result<[crunch_radiance_reference_core::RadianceArtifactObservation; 2], crate::errors::RunError> {
    let (crt_digest_blake3, crt_byte_count) =
        crate::source_bundle::observe_bound_foreign_source_path(CRT_INPUTS_PAYLOAD_ID, crt_dir)?;
    let (libgcc_digest_blake3, libgcc_byte_count) =
        crate::source_bundle::observe_bound_foreign_source_path(LIBGCC_INPUTS_PAYLOAD_ID, libgcc_dir)?;
    if crt_byte_count == 0 || libgcc_byte_count == 0 {
        return Err(crate::errors::RunError::Internal("Radiance link runtime input projection is empty".to_string()));
    }
    let observations = [
        crunch_radiance_reference_core::RadianceArtifactObservation {
            role: crunch_radiance_reference_core::RadianceArtifactRole::HostCrtInputs,
            digest_blake3: crt_digest_blake3,
            byte_count: crt_byte_count,
        },
        crunch_radiance_reference_core::RadianceArtifactObservation {
            role: crunch_radiance_reference_core::RadianceArtifactRole::HostLibgccInputs,
            digest_blake3: libgcc_digest_blake3,
            byte_count: libgcc_byte_count,
        },
    ];
    debug_assert!(observations.iter().all(|observation| observation.byte_count > 0));
    debug_assert_ne!(observations[0].role, observations[1].role);
    Ok(observations)
}

pub(super) fn require_link_inputs(
    crt_dir: &std::path::Path,
    libgcc_dir: &std::path::Path,
) -> Result<(), crate::errors::RunError> {
    require_input_dir(crt_dir, "CRT", CRT_REQUIRED_FILES)?;
    require_input_dir(libgcc_dir, "libgcc", LIBGCC_REQUIRED_FILES)?;
    debug_assert!(crt_dir.is_absolute());
    debug_assert!(libgcc_dir.is_absolute());
    Ok(())
}

fn require_input_dir(
    path: &std::path::Path,
    label: &str,
    required_files: &[&str],
) -> Result<(), crate::errors::RunError> {
    if !path.is_absolute() || !path.is_dir() {
        return Err(crate::errors::RunError::Internal(format!(
            "Radiance {label} input is not an absolute directory: {}",
            path.display()
        )));
    }
    for name in required_files {
        let input = path.join(name);
        if !input.is_file() {
            return Err(crate::errors::RunError::Internal(format!(
                "Radiance {label} input is missing required file {}",
                input.display()
            )));
        }
    }
    debug_assert!(!required_files.is_empty());
    debug_assert!(required_files.iter().all(|name| path.join(name).is_file()));
    Ok(())
}

pub(super) fn validate_source_argument(source: &str) -> Result<(), crate::errors::RunError> {
    let path = std::path::Path::new(source);
    let is_safe = !source.is_empty()
        && !path.is_absolute()
        && path.components().all(|component| matches!(component, std::path::Component::Normal(_)));
    if !is_safe {
        return Err(crate::errors::RunError::Internal(format!("unsafe Radiance native source argument: {source}")));
    }
    debug_assert!(!source.is_empty());
    debug_assert!(!path.is_absolute());
    Ok(())
}

pub(super) fn observe_artifact(
    path: &std::path::Path,
    role: crunch_radiance_reference_core::RadianceArtifactRole,
    label: &str,
) -> Result<crunch_radiance_reference_core::RadianceArtifactObservation, crate::errors::RunError> {
    let (digest_blake3, byte_count) = crate::radiance::source::digest_file(path, label)?;
    debug_assert!(!digest_blake3.is_empty());
    debug_assert!(byte_count > 0);
    Ok(crunch_radiance_reference_core::RadianceArtifactObservation {
        role,
        digest_blake3,
        byte_count,
    })
}
