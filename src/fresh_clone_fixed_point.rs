use serde::Deserialize;
use serde::Serialize;

pub const HYDRATED_FRESH_CLONE_FIXED_POINT_FORMAT: &str = "mantle-hydrated-fresh-clone-fixed-point-v1";
pub const HYDRATED_FRESH_CLONE_FIXED_POINT_VERSION: u32 = 1;
pub const HYDRATED_FRESH_CLONE_FIXED_POINT_NON_CLAIM: &str = "This report proves one hydrated legacy-provider fixed point for the recorded source authority and platform; it does not prove full-source bootstrap, compiler correctness, seed trust removal, release reproducibility, independent rebuild agreement, deployment success, or full Cargo compatibility.";
const BLAKE3_HEX_LENGTH: usize = 64;

// machine-artifact-public: self-build.hydrated-fresh-clone-fixed-point-report
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HydratedFreshCloneFixedPointReport {
    pub format: String,
    pub version: u32,
    pub expected_manifest_blake3: String,
    pub source_state_blake3: String,
    pub hydration_report_blake3: String,
    pub staged_source_store_name: String,
    pub provider_kind: String,
    pub platform: String,
    pub proof_mode: String,
    pub stage0: HydratedFreshCloneStageReport,
    pub stage2: HydratedFreshCloneStageReport,
    pub stage1_binary_blake3: String,
    pub stage2_binary_blake3: String,
    pub fixed_point: bool,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HydratedFreshCloneStageReport {
    pub source_policy: String,
    pub source_override_count: u32,
    pub live_fetch_events: u32,
    pub hermeticity_mode: String,
    pub fallback_event_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HydratedFreshCloneReportInput {
    pub expected_manifest_blake3: String,
    pub source_state_blake3: String,
    pub hydration_report_blake3: String,
    pub staged_source_store_name: String,
    pub provider_kind: String,
    pub platform: String,
    pub proof_mode: String,
    pub stage0: HydratedFreshCloneStageReport,
    pub stage2: HydratedFreshCloneStageReport,
    pub stage1_binary_blake3: String,
    pub stage2_binary_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StringFieldValidation<'a> {
    field: &'static str,
    value: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HydratedFreshCloneReportError {
    InvalidBlake3 { field: &'static str },
    EmptyField { field: &'static str },
    SourcePolicyNotEnforced { stage: &'static str },
    EmptySourceOverrides { stage: &'static str },
    LiveFetchObserved { stage: &'static str },
    UnsafeStagedSourceStoreName,
}

impl std::fmt::Display for HydratedFreshCloneReportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidBlake3 { field } => write!(formatter, "{field} must be 64-character lowercase BLAKE3 hex"),
            Self::EmptyField { field } => write!(formatter, "{field} must not be empty"),
            Self::SourcePolicyNotEnforced { stage } => {
                write!(formatter, "{stage} source policy must be require-override")
            }
            Self::EmptySourceOverrides { stage } => write!(formatter, "{stage} source override count must be nonzero"),
            Self::LiveFetchObserved { stage } => write!(formatter, "{stage} live fetch event count must be zero"),
            Self::UnsafeStagedSourceStoreName => {
                write!(formatter, "staged_source_store_name must be a path-free store entry name")
            }
        }
    }
}

impl std::error::Error for HydratedFreshCloneReportError {}

impl HydratedFreshCloneFixedPointReport {
    pub fn from_input(input: HydratedFreshCloneReportInput) -> Result<Self, HydratedFreshCloneReportError> {
        validate_blake3(StringFieldValidation {
            field: "expected_manifest_blake3",
            value: &input.expected_manifest_blake3,
        })?;
        validate_blake3(StringFieldValidation {
            field: "source_state_blake3",
            value: &input.source_state_blake3,
        })?;
        validate_blake3(StringFieldValidation {
            field: "hydration_report_blake3",
            value: &input.hydration_report_blake3,
        })?;
        validate_blake3(StringFieldValidation {
            field: "stage1_binary_blake3",
            value: &input.stage1_binary_blake3,
        })?;
        validate_blake3(StringFieldValidation {
            field: "stage2_binary_blake3",
            value: &input.stage2_binary_blake3,
        })?;
        validate_staged_source_store_name(&input.staged_source_store_name)?;
        validate_non_empty(StringFieldValidation {
            field: "provider_kind",
            value: &input.provider_kind,
        })?;
        validate_non_empty(StringFieldValidation {
            field: "platform",
            value: &input.platform,
        })?;
        validate_non_empty(StringFieldValidation {
            field: "proof_mode",
            value: &input.proof_mode,
        })?;
        validate_stage("stage0", &input.stage0)?;
        validate_stage("stage2", &input.stage2)?;
        let is_fixed_point = input.stage1_binary_blake3 == input.stage2_binary_blake3;
        Ok(Self {
            format: HYDRATED_FRESH_CLONE_FIXED_POINT_FORMAT.to_string(),
            version: HYDRATED_FRESH_CLONE_FIXED_POINT_VERSION,
            expected_manifest_blake3: input.expected_manifest_blake3,
            source_state_blake3: input.source_state_blake3,
            hydration_report_blake3: input.hydration_report_blake3,
            staged_source_store_name: input.staged_source_store_name,
            provider_kind: input.provider_kind,
            platform: input.platform,
            proof_mode: input.proof_mode,
            stage0: input.stage0,
            stage2: input.stage2,
            stage1_binary_blake3: input.stage1_binary_blake3,
            stage2_binary_blake3: input.stage2_binary_blake3,
            fixed_point: is_fixed_point,
            non_claim: HYDRATED_FRESH_CLONE_FIXED_POINT_NON_CLAIM.to_string(),
        })
    }
}

fn validate_blake3(request: StringFieldValidation<'_>) -> Result<(), HydratedFreshCloneReportError> {
    let has_expected_length_bytes = request.value.len() == BLAKE3_HEX_LENGTH;
    let is_lowercase_hex = request.value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
    if !has_expected_length_bytes || !is_lowercase_hex {
        return Err(HydratedFreshCloneReportError::InvalidBlake3 { field: request.field });
    }
    Ok(())
}

fn validate_staged_source_store_name(value: &str) -> Result<(), HydratedFreshCloneReportError> {
    validate_non_empty(StringFieldValidation {
        field: "staged_source_store_name",
        value,
    })?;
    if value.contains('/') {
        return Err(HydratedFreshCloneReportError::UnsafeStagedSourceStoreName);
    }
    if value.contains('\\') {
        return Err(HydratedFreshCloneReportError::UnsafeStagedSourceStoreName);
    }
    if value == "." {
        return Err(HydratedFreshCloneReportError::UnsafeStagedSourceStoreName);
    }
    if value == ".." {
        return Err(HydratedFreshCloneReportError::UnsafeStagedSourceStoreName);
    }
    Ok(())
}

fn validate_non_empty(request: StringFieldValidation<'_>) -> Result<(), HydratedFreshCloneReportError> {
    if request.value.is_empty() {
        return Err(HydratedFreshCloneReportError::EmptyField { field: request.field });
    }
    Ok(())
}

fn validate_stage(
    stage: &'static str,
    report: &HydratedFreshCloneStageReport,
) -> Result<(), HydratedFreshCloneReportError> {
    if report.source_policy != "require-override" {
        return Err(HydratedFreshCloneReportError::SourcePolicyNotEnforced { stage });
    }
    if report.source_override_count == 0 {
        return Err(HydratedFreshCloneReportError::EmptySourceOverrides { stage });
    }
    if report.live_fetch_events != 0 {
        return Err(HydratedFreshCloneReportError::LiveFetchObserved { stage });
    }
    validate_non_empty(StringFieldValidation {
        field: "hermeticity_mode",
        value: &report.hermeticity_mode,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE_OVERRIDE_COUNT: u32 = 12;

    fn stage() -> HydratedFreshCloneStageReport {
        HydratedFreshCloneStageReport {
            source_policy: "require-override".to_string(),
            source_override_count: SOURCE_OVERRIDE_COUNT,
            live_fetch_events: 0,
            hermeticity_mode: "strict".to_string(),
            fallback_event_count: 0,
        }
    }

    fn input() -> HydratedFreshCloneReportInput {
        HydratedFreshCloneReportInput {
            expected_manifest_blake3: "a".repeat(BLAKE3_HEX_LENGTH),
            source_state_blake3: "b".repeat(BLAKE3_HEX_LENGTH),
            hydration_report_blake3: "c".repeat(BLAKE3_HEX_LENGTH),
            staged_source_store_name: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-mantle-src".to_string(),
            provider_kind: "musl.cc-native-reduced-v1".to_string(),
            platform: "x86_64-linux".to_string(),
            proof_mode: "fixed-point".to_string(),
            stage0: stage(),
            stage2: stage(),
            stage1_binary_blake3: "d".repeat(BLAKE3_HEX_LENGTH),
            stage2_binary_blake3: "d".repeat(BLAKE3_HEX_LENGTH),
        }
    }

    #[test]
    fn report_admits_matching_offline_stages_and_binary_digests() {
        let report = HydratedFreshCloneFixedPointReport::from_input(input()).unwrap();

        assert!(report.fixed_point);
        assert_eq!(report.format, HYDRATED_FRESH_CLONE_FIXED_POINT_FORMAT);
        assert_eq!(report.stage0.live_fetch_events, 0);
        assert_eq!(report.stage2.source_override_count, SOURCE_OVERRIDE_COUNT);
    }

    #[test]
    fn report_rejects_live_fetch_event() {
        let mut input = input();
        input.stage2.live_fetch_events = 1;

        let error = HydratedFreshCloneFixedPointReport::from_input(input).unwrap_err();

        assert_eq!(error, HydratedFreshCloneReportError::LiveFetchObserved { stage: "stage2" });
    }

    #[test]
    fn report_rejects_staged_source_path_leak() {
        let mut input = input();
        input.staged_source_store_name = "/tmp/producer/mantle-src".to_string();

        let error = HydratedFreshCloneFixedPointReport::from_input(input).unwrap_err();

        assert_eq!(error, HydratedFreshCloneReportError::UnsafeStagedSourceStoreName);
    }

    #[test]
    fn report_records_mismatched_binary_digests_without_admitting_fixed_point() {
        let mut input = input();
        input.stage2_binary_blake3 = "e".repeat(BLAKE3_HEX_LENGTH);

        let report = HydratedFreshCloneFixedPointReport::from_input(input).unwrap();

        assert!(!report.fixed_point);
        assert_ne!(report.stage1_binary_blake3, report.stage2_binary_blake3);
    }
}
