const GIT_OUTPUT_BYTES_MAX: usize = 65_536;
const CHECKOUT_COUNT: usize = 3;

#[derive(Debug, Clone)]
pub(crate) struct PrepareRequest<'a> {
    pub git: &'a std::path::Path,
    pub radiance_checkout: &'a std::path::Path,
    pub bootstrap_compiler_checkout: &'a std::path::Path,
    pub emulator_checkout: &'a std::path::Path,
    pub source_bundle_out: &'a std::path::Path,
    pub cohort_out: &'a std::path::Path,
    pub store_prefix: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct PrepareReport {
    pub schema: &'static str,
    pub source_bundle_blake3: String,
    pub source_cohort_blake3: String,
    pub source_count: u32,
    pub git_blake3: String,
    pub source_bundle_path: String,
    pub cohort_path: String,
    pub network_required_for_proof: bool,
    pub non_claim: &'static str,
}

pub(crate) struct PreparedInputs {
    pub manifest: crate::source_bundle::SourceBundleManifest,
    pub cohort: crunch_radiance_reference_core::RadianceSourceCohortWire,
}

pub(crate) fn prepare(request: PrepareRequest<'_>) -> Result<PrepareReport, crate::errors::RunError> {
    let profile = crate::radiance::profile::load()?;
    let prepared = collect_inputs(&profile, &request)?;
    crate::source_bundle::write_source_bundle_no_replace(request.source_bundle_out, &prepared.manifest)?;
    let cohort_bytes = canonical_pretty_json(&prepared.cohort, "Radiance source cohort")?;
    crate::source_bundle::publish_immutable_source_bytes(request.cohort_out, &cohort_bytes, "Radiance source cohort")?;
    let prepare_result = PrepareReport {
        schema: "mantle-radiance-reference-prepare-report-v1",
        source_bundle_blake3: prepared.manifest.manifest_blake3,
        source_cohort_blake3: prepared.cohort.cohort_blake3,
        source_count: u32::try_from(prepared.cohort.members.len())
            .map_err(|_| crate::errors::RunError::Internal("Radiance source count overflow".to_string()))?,
        git_blake3: digest_file(request.git, "Git executable")?.0,
        source_bundle_path: request.source_bundle_out.display().to_string(),
        cohort_path: request.cohort_out.display().to_string(),
        network_required_for_proof: false,
        non_claim: crunch_radiance_reference_core::RADIANCE_REFERENCE_NON_CLAIM,
    };
    debug_assert_eq!(usize::try_from(prepare_result.source_count).ok(), Some(CHECKOUT_COUNT));
    debug_assert!(!prepare_result.source_bundle_blake3.is_empty());
    Ok(prepare_result)
}

pub(crate) fn collect_inputs(
    profile: &crate::radiance::profile::Definition,
    request: &PrepareRequest<'_>,
) -> Result<PreparedInputs, crate::errors::RunError> {
    require_absolute_executable(request.git, "Git executable")?;
    let checkouts = [
        (crunch_radiance_reference_core::RadianceSourceRole::Radiance, request.radiance_checkout),
        (
            crunch_radiance_reference_core::RadianceSourceRole::BootstrapCompiler,
            request.bootstrap_compiler_checkout,
        ),
        (crunch_radiance_reference_core::RadianceSourceRole::Emulator, request.emulator_checkout),
    ];
    for (role, checkout) in checkouts {
        validate_checkout(request.git, checkout, role)?;
    }
    let specs = checkouts
        .iter()
        .map(|(role, checkout)| crate::source_bundle::VcsCheckoutSourceSpec {
            identity: crate::radiance::profile::role_label(*role).to_string(),
            path: (*checkout).to_path_buf(),
            repository_url: role.expected_repository_url().to_string(),
            revision: role.expected_revision().to_string(),
        })
        .collect::<Vec<_>>();
    let manifest = crate::source_bundle::plan_vcs_checkout_source_bundle(&specs, request.store_prefix)?;
    let members = manifest
        .records
        .iter()
        .map(|record| member_from_record(record, profile))
        .collect::<Result<Vec<_>, _>>()?;
    let cohort = crunch_radiance_reference_core::admit_source_cohort(members)
        .map_err(|error| crate::errors::RunError::Internal(format!("admitting Radiance source cohort: {error:?}")))?
        .into_wire();
    debug_assert_eq!(manifest.records.len(), CHECKOUT_COUNT);
    debug_assert_eq!(cohort.members.len(), CHECKOUT_COUNT);
    Ok(PreparedInputs { manifest, cohort })
}

pub(super) fn validate_checkout(
    git: &std::path::Path,
    checkout: &std::path::Path,
    role: crunch_radiance_reference_core::RadianceSourceRole,
) -> Result<(), crate::errors::RunError> {
    if !checkout.is_absolute() || !checkout.is_dir() {
        return Err(crate::errors::RunError::Internal(format!(
            "Radiance {} checkout is not an absolute directory",
            crate::radiance::profile::role_label(role)
        )));
    }
    let object_format = git_text(git, checkout, &["rev-parse", "--show-object-format"])?;
    if object_format != "sha256" {
        return Err(crate::errors::RunError::Internal(format!(
            "Radiance {} checkout uses Git object format {object_format}",
            crate::radiance::profile::role_label(role)
        )));
    }
    let revision = git_text(git, checkout, &["rev-parse", "HEAD"])?;
    if revision != role.expected_revision() {
        return Err(crate::errors::RunError::Internal(format!(
            "Radiance {} checkout revision drifted: expected {}, got {revision}",
            crate::radiance::profile::role_label(role),
            role.expected_revision()
        )));
    }
    let status = git_text(git, checkout, &["status", "--porcelain=v1", "--untracked-files=all"])?;
    if !status.is_empty() {
        return Err(crate::errors::RunError::Internal(format!(
            "Radiance {} checkout is dirty",
            crate::radiance::profile::role_label(role)
        )));
    }
    let license = checkout.join("LICENSE");
    let (license_blake3, _) = digest_file(&license, "Radiance license")?;
    if license_blake3 != crunch_radiance_reference_core::RADIANCE_MIT_LICENSE_BLAKE3 {
        return Err(crate::errors::RunError::Internal(format!(
            "Radiance {} MIT license bytes drifted",
            crate::radiance::profile::role_label(role)
        )));
    }
    debug_assert_eq!(object_format, "sha256");
    debug_assert_eq!(revision, role.expected_revision());
    Ok(())
}

fn member_from_record(
    record: &crate::source_bundle::SourceRecord,
    profile: &crate::radiance::profile::Definition,
) -> Result<crunch_radiance_reference_core::RadianceSourceMemberWire, crate::errors::RunError> {
    let role = role_from_label(&record.identity)?;
    let profile_row = profile.sources.iter().find(|row| row.role == record.identity).ok_or_else(|| {
        crate::errors::RunError::Internal(format!("Radiance profile has no source role {}", record.identity))
    })?;
    let observation = crate::source_bundle::admitted_source_observation(record)?.ok_or_else(|| {
        crate::errors::RunError::Internal(format!(
            "Radiance source {} did not produce a complete observation",
            record.identity
        ))
    })?;
    if observation.content_blake3 != profile_row.content_blake3 {
        return Err(crate::errors::RunError::Internal(format!(
            "Radiance source content drifted for {}",
            record.identity
        )));
    }
    debug_assert_eq!(profile_row.revision, role.expected_revision());
    debug_assert_eq!(profile_row.repository_url, role.expected_repository_url());
    Ok(crunch_radiance_reference_core::RadianceSourceMemberWire {
        role,
        repository_url: profile_row.repository_url.clone(),
        observation,
        license_spdx: profile_row.license_spdx.clone(),
        license_blake3: profile_row.license_blake3.clone(),
    })
}

pub(super) fn role_from_label(
    value: &str,
) -> Result<crunch_radiance_reference_core::RadianceSourceRole, crate::errors::RunError> {
    match value {
        "radiance" => Ok(crunch_radiance_reference_core::RadianceSourceRole::Radiance),
        "bootstrap-compiler" => Ok(crunch_radiance_reference_core::RadianceSourceRole::BootstrapCompiler),
        "emulator" => Ok(crunch_radiance_reference_core::RadianceSourceRole::Emulator),
        other => Err(crate::errors::RunError::Internal(format!("unsupported Radiance source role {other}"))),
    }
}

fn git_text(
    git: &std::path::Path,
    checkout: &std::path::Path,
    arguments: &[&str],
) -> Result<String, crate::errors::RunError> {
    let output = std::process::Command::new(git)
        .args(arguments)
        .current_dir(checkout)
        .env_clear()
        .output()
        .map_err(|error| crate::errors::RunError::Internal(format!("running explicit Git executable: {error}")))?;
    if output.stdout.len() > GIT_OUTPUT_BYTES_MAX || output.stderr.len() > GIT_OUTPUT_BYTES_MAX {
        return Err(crate::errors::RunError::Internal("Git source observation output exceeded its bound".to_string()));
    }
    if !output.status.success() {
        return Err(crate::errors::RunError::Internal(format!(
            "Git source observation failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let text = String::from_utf8(output.stdout)
        .map_err(|_| crate::errors::RunError::Internal("Git source observation output is not UTF-8".to_string()))?;
    debug_assert!(output.status.success());
    debug_assert!(text.len() <= GIT_OUTPUT_BYTES_MAX);
    Ok(text.trim().to_string())
}

pub(super) fn require_absolute_executable(path: &std::path::Path, label: &str) -> Result<(), crate::errors::RunError> {
    if !path.is_absolute() || !path.is_file() {
        return Err(crate::errors::RunError::Internal(format!(
            "{label} is not an absolute regular file: {}",
            path.display()
        )));
    }
    debug_assert!(path.is_absolute());
    debug_assert!(path.is_file());
    Ok(())
}

pub(super) fn digest_file(path: &std::path::Path, label: &str) -> Result<(String, u64), crate::errors::RunError> {
    let bytes = std::fs::read(path)
        .map_err(|error| crate::errors::RunError::Internal(format!("reading {label} {}: {error}", path.display())))?;
    let byte_count = u64::try_from(bytes.len())
        .map_err(|_| crate::errors::RunError::Internal(format!("{label} byte count overflow")))?;
    if byte_count == 0 || byte_count > crunch_radiance_reference_core::RADIANCE_ARTIFACT_BYTES_MAX {
        return Err(crate::errors::RunError::Internal(format!("{label} byte count is outside the admitted bound")));
    }
    let digest = blake3::hash(&bytes).to_hex().to_string();
    debug_assert_eq!(digest.len(), crunch_radiance_reference_core::BLAKE3_HEX_CHARS);
    debug_assert!(byte_count > 0);
    Ok((digest, byte_count))
}

pub(super) fn canonical_pretty_json<T: serde::Serialize>(
    value: &T,
    label: &str,
) -> Result<Vec<u8>, crate::errors::RunError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| crate::errors::RunError::Internal(format!("serializing {label}: {error}")))?;
    bytes.push(b'\n');
    debug_assert!(!bytes.is_empty());
    debug_assert_eq!(bytes.last(), Some(&b'\n'));
    Ok(bytes)
}
