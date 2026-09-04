pub(super) const PROFILE_SCHEMA: &str = "mantle-radiance-reference-profile-v1";
pub(super) const PROFILE_NON_CLAIM: &str = "external reference convergence does not prove compiler correctness, seed trust, semantic equivalence, or universal reproducibility";
const PROFILE_JSON: &str = include_str!("../../config/generated/radiance-reference.json");
const EMULATOR_ARGUMENT_COUNT: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Definition {
    pub schema: String,
    pub sources: Vec<SourceRow>,
    pub native_builds: NativeBuildSet,
    pub emulator_arguments: Vec<String>,
    pub compiler_arguments: Vec<String>,
    pub expected_fixed_point_blake3: String,
    pub expected_fixed_point_bytes: u64,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceRow {
    pub role: String,
    pub repository_url: String,
    pub revision: String,
    pub content_blake3: String,
    pub projection: String,
    pub snapshot_profile: String,
    pub license_spdx: String,
    pub license_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativeBuildSet {
    pub bootstrap_compiler: NativeBuild,
    pub emulator: NativeBuild,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativeBuild {
    pub role: String,
    pub sources: Vec<String>,
    pub flags: Vec<String>,
    pub linker_flags: Vec<String>,
    pub output_name: String,
}

pub(super) fn load() -> Result<Definition, crate::errors::RunError> {
    let profile: Definition = serde_json::from_str(PROFILE_JSON).map_err(|error| {
        crate::errors::RunError::Internal(format!("parsing embedded Radiance reference profile: {error}"))
    })?;
    validate(&profile)?;
    debug_assert_eq!(profile.schema, PROFILE_SCHEMA);
    debug_assert_eq!(profile.sources.len(), crunch_radiance_reference_core::RADIANCE_SOURCE_COUNT);
    Ok(profile)
}

pub(super) fn validate(profile: &Definition) -> Result<(), crate::errors::RunError> {
    if profile.schema != PROFILE_SCHEMA || profile.non_claim != PROFILE_NON_CLAIM {
        return Err(crate::errors::RunError::Internal("Radiance reference profile header drifted".to_string()));
    }
    if profile.sources.len() != crunch_radiance_reference_core::RADIANCE_SOURCE_COUNT {
        return Err(crate::errors::RunError::Internal("Radiance reference source count drifted".to_string()));
    }
    if profile.emulator_arguments.len() != EMULATOR_ARGUMENT_COUNT || profile.compiler_arguments.is_empty() {
        return Err(crate::errors::RunError::Internal("Radiance reference stage arguments drifted".to_string()));
    }
    validate_sources(&profile.sources)?;
    validate_builds(&profile.native_builds)?;
    if profile.expected_fixed_point_blake3 != "a06905539bd81c9581218ca648e98fb7a55c5a7e79847b6c840181a3e2605b07" {
        return Err(crate::errors::RunError::Internal("Radiance expected fixed-point identity drifted".to_string()));
    }
    if profile.expected_fixed_point_bytes == 0 {
        return Err(crate::errors::RunError::Internal("Radiance expected fixed-point byte count is zero".to_string()));
    }
    debug_assert!(!profile.compiler_arguments.is_empty());
    debug_assert!(profile.expected_fixed_point_bytes > 0);
    Ok(())
}

fn validate_sources(rows: &[SourceRow]) -> Result<(), crate::errors::RunError> {
    let expected = [
        crunch_radiance_reference_core::RadianceSourceRole::Radiance,
        crunch_radiance_reference_core::RadianceSourceRole::BootstrapCompiler,
        crunch_radiance_reference_core::RadianceSourceRole::Emulator,
    ];
    for (row, role) in rows.iter().zip(expected) {
        if row.role != role_label(role) {
            return Err(crate::errors::RunError::Internal("Radiance reference source role drifted".to_string()));
        }
        if row.repository_url != role.expected_repository_url()
            || row.revision != role.expected_revision()
            || row.content_blake3 != role.expected_content_blake3()
        {
            return Err(crate::errors::RunError::Internal(format!(
                "Radiance reference source identity drifted for {}",
                row.role
            )));
        }
        if row.projection != "." || row.snapshot_profile != "canonical-tree-v1" {
            return Err(crate::errors::RunError::Internal(format!(
                "Radiance source projection drifted for {}",
                row.role
            )));
        }
        if row.license_spdx != crunch_radiance_reference_core::RADIANCE_LICENSE_SPDX
            || row.license_blake3 != crunch_radiance_reference_core::RADIANCE_MIT_LICENSE_BLAKE3
        {
            return Err(crate::errors::RunError::Internal(format!("Radiance source license drifted for {}", row.role)));
        }
    }
    debug_assert_eq!(rows.len(), expected.len());
    debug_assert!(rows.iter().all(|row| !row.role.is_empty()));
    Ok(())
}

fn validate_builds(rows: &NativeBuildSet) -> Result<(), crate::errors::RunError> {
    let expected = [
        (&rows.bootstrap_compiler, "bootstrap-compiler"),
        (&rows.emulator, "emulator"),
    ];
    for (row, role) in expected {
        if row.role != role {
            return Err(crate::errors::RunError::Internal(format!(
                "Radiance native build role is invalid: {}",
                row.role
            )));
        }
        if row.sources.is_empty() {
            return Err(crate::errors::RunError::Internal(format!(
                "Radiance native build sources are empty: {}",
                row.role
            )));
        }
        if row.flags.is_empty() {
            return Err(crate::errors::RunError::Internal(format!(
                "Radiance native build flags are empty: {}",
                row.role
            )));
        }
        if row.linker_flags.is_empty() {
            return Err(crate::errors::RunError::Internal(format!(
                "Radiance native linker flags are empty: {}",
                row.role
            )));
        }
        if row.output_name.is_empty() {
            return Err(crate::errors::RunError::Internal(format!(
                "Radiance native output name is empty: {}",
                row.role
            )));
        }
    }
    debug_assert!(!rows.bootstrap_compiler.sources.is_empty());
    debug_assert!(!rows.emulator.sources.is_empty());
    Ok(())
}

pub(super) const fn role_label(role: crunch_radiance_reference_core::RadianceSourceRole) -> &'static str {
    match role {
        crunch_radiance_reference_core::RadianceSourceRole::Radiance => "radiance",
        crunch_radiance_reference_core::RadianceSourceRole::BootstrapCompiler => "bootstrap-compiler",
        crunch_radiance_reference_core::RadianceSourceRole::Emulator => "emulator",
    }
}
