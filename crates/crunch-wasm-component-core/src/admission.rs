use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::AotMode;
use crate::Blake3Identity;
use crate::ComponentBlocker;
use crate::StoreObject;
use crate::WizerMode;
use crate::blocker::blocker;
use crate::digest::canonical_identity;
use crate::digest::is_count_above_bound;

pub const PORTABLE_ADMISSION_SCHEMA: &str = "mantle-wasm-component-portable-admission-v1";
pub const WIZER_ADMISSION_SCHEMA: &str = "mantle-wasm-component-wizer-admission-v1";
pub const AOT_ADMISSION_SCHEMA: &str = "mantle-wasm-component-aot-admission-v1";
pub const AOT_TRUST_CLASS: &str = "target-specific-trusted-native-code";

const MAX_TRANSFORM_IMPORTS: u32 = 256;
const MAX_AMBIENT_OBSERVATIONS: u32 = 64;
const MAX_CPU_FEATURES: u32 = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationDecision {
    Pass,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildValidationBinding {
    pub artifact_blake3: Blake3Identity,
    pub cohort_blake3: Blake3Identity,
    pub report_blake3: Blake3Identity,
    pub decision: ValidationDecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OctetValidationBinding {
    pub artifact_blake3: Blake3Identity,
    pub profile_blake3: Blake3Identity,
    pub cohort_blake3: Blake3Identity,
    pub report_blake3: Blake3Identity,
    pub decision: ValidationDecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortableAdmissionRequest {
    pub artifact: StoreObject,
    pub build_validation: BuildValidationBinding,
    pub octet_validation: Option<OctetValidationBinding>,
    pub octet_required: bool,
    pub expected_octet_profile_blake3: Blake3Identity,
    pub expected_octet_cohort_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortableAdmission {
    pub schema: String,
    pub artifact: StoreObject,
    pub build_validation_report_blake3: Blake3Identity,
    pub octet_validation_report_blake3: Option<Blake3Identity>,
    pub admitted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortableAdmissionResult {
    pub admission: Option<PortableAdmission>,
    pub blockers: Vec<ComponentBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformImportFact {
    pub name: String,
    pub deterministic: bool,
    pub input_identity_blake3: Option<Blake3Identity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformAdmissionRequest {
    pub mode: WizerMode,
    pub input: StoreObject,
    pub first_output: StoreObject,
    pub repeated_output: Option<StoreObject>,
    pub cohort_blake3: Blake3Identity,
    pub initialization_entrypoint: String,
    pub virtual_imports: Vec<TransformImportFact>,
    pub ambient_observations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformAdmission {
    pub schema: String,
    pub input_blake3: Blake3Identity,
    pub output: StoreObject,
    pub cohort_blake3: Blake3Identity,
    pub deterministic_outputs_match: bool,
    pub eligible_for_bundle: bool,
    pub receipt_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformAdmissionResult {
    pub admission: Option<TransformAdmission>,
    pub blockers: Vec<ComponentBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AotReceipt {
    pub source_component_blake3: Blake3Identity,
    pub output: StoreObject,
    pub target: String,
    pub cpu_features: Vec<String>,
    pub wasmtime_configuration_blake3: Blake3Identity,
    pub cohort_blake3: Blake3Identity,
    pub wit_profile_blake3: Blake3Identity,
    pub build_inputs_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AotAdmissionRequest {
    pub mode: AotMode,
    pub portable_admission: PortableAdmission,
    pub receipt: AotReceipt,
    pub expected_target: String,
    pub expected_cpu_features: Vec<String>,
    pub expected_wasmtime_configuration_blake3: Blake3Identity,
    pub expected_cohort_blake3: Blake3Identity,
    pub expected_wit_profile_blake3: Blake3Identity,
    pub expected_build_inputs_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AotAdmission {
    pub schema: String,
    pub source_component_blake3: Blake3Identity,
    pub output: StoreObject,
    pub target: String,
    pub cpu_features: Vec<String>,
    pub trust_class: String,
    pub wasmtime_configuration_blake3: Blake3Identity,
    pub cohort_blake3: Blake3Identity,
    pub wit_profile_blake3: Blake3Identity,
    pub build_inputs_blake3: Blake3Identity,
    pub receipt_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AotAdmissionResult {
    pub admission: Option<AotAdmission>,
    pub blockers: Vec<ComponentBlocker>,
}

pub fn bind_portable_admission(request: PortableAdmissionRequest) -> PortableAdmissionResult {
    let mut blockers = Vec::new();
    if request.artifact.size_bytes == 0 || !request.artifact.logical_path.starts_with('/') {
        blockers.push(blocker(
            "invalid-portable-artifact",
            "portable-component",
            "portable artifact must be an identified non-empty store object",
        ));
    }
    if request.build_validation.artifact_blake3 != request.artifact.digest_blake3 {
        blockers.push(blocker(
            "build-validation-artifact-mismatch",
            "wasm-tools-report",
            "build validation report names different portable bytes",
        ));
    }
    if request.build_validation.decision != ValidationDecision::Pass {
        blockers.push(blocker(
            "build-validation-failed",
            "wasm-tools-report",
            "portable artifact did not pass build-local validation",
        ));
    }
    validate_octet_binding(&request, &mut blockers);
    if !blockers.is_empty() {
        return PortableAdmissionResult {
            admission: None,
            blockers,
        };
    }
    // A digest is an identity, not a numeric quantity.
    #[allow(tigerstyle::numeric_units)]
    let octet_report_digest = request.octet_validation.map(|binding| binding.report_blake3);
    debug_assert!(!request.octet_required || octet_report_digest.is_some());
    debug_assert_eq!(request.build_validation.artifact_blake3, request.artifact.digest_blake3);
    PortableAdmissionResult {
        admission: Some(PortableAdmission {
            schema: String::from(PORTABLE_ADMISSION_SCHEMA),
            artifact: request.artifact,
            build_validation_report_blake3: request.build_validation.report_blake3,
            octet_validation_report_blake3: octet_report_digest,
            admitted: true,
        }),
        blockers: Vec::new(),
    }
}

pub fn admit_transform(request: TransformAdmissionRequest) -> TransformAdmissionResult {
    let mut blockers = Vec::new();
    validate_transform_request(&request, &mut blockers);
    let is_deterministic_output_match = repeated_output_matches(&request);
    if request.mode == WizerMode::Deterministic && !is_deterministic_output_match {
        blockers.push(blocker(
            "wizer-output-drift",
            "wizer",
            "repeated clean Wizer outputs do not have the same BLAKE3 identity",
        ));
    }
    if !blockers.is_empty() {
        return TransformAdmissionResult {
            admission: None,
            blockers,
        };
    }
    let is_eligible_for_bundle = request.mode == WizerMode::Deterministic;
    let receipt_identity = match canonical_identity(request.clone()) {
        Ok(identity) => identity,
        Err(_) => {
            return TransformAdmissionResult {
                admission: None,
                blockers: vec![blocker(
                    "wizer-receipt-identity-failed",
                    "wizer",
                    "Wizer admission receipt could not be canonically identified",
                )],
            };
        }
    };
    debug_assert!(request.mode != WizerMode::Disabled);
    debug_assert!(!is_eligible_for_bundle || is_deterministic_output_match);
    TransformAdmissionResult {
        admission: Some(TransformAdmission {
            schema: String::from(WIZER_ADMISSION_SCHEMA),
            input_blake3: request.input.digest_blake3,
            output: request.first_output,
            cohort_blake3: request.cohort_blake3,
            deterministic_outputs_match: is_deterministic_output_match,
            eligible_for_bundle: is_eligible_for_bundle,
            receipt_blake3: receipt_identity,
        }),
        blockers: Vec::new(),
    }
}

pub fn admit_aot(mut request: AotAdmissionRequest) -> AotAdmissionResult {
    let mut blockers = Vec::new();
    validate_aot_request(&request, &mut blockers);
    normalize_cpu_features(&mut request.expected_cpu_features, &mut blockers, "expected-cpu-features");
    normalize_cpu_features(&mut request.receipt.cpu_features, &mut blockers, "receipt-cpu-features");
    if request.expected_cpu_features != request.receipt.cpu_features {
        blockers.push(blocker(
            "aot-cpu-feature-mismatch",
            "aot-receipt",
            "precompiled artifact CPU features differ from the expected release target",
        ));
    }
    if !blockers.is_empty() {
        return AotAdmissionResult {
            admission: None,
            blockers,
        };
    }
    let receipt_identity = match canonical_identity(request.receipt.clone()) {
        Ok(identity) => identity,
        Err(_) => {
            return AotAdmissionResult {
                admission: None,
                blockers: vec![blocker(
                    "aot-receipt-identity-failed",
                    "aot-receipt",
                    "AOT receipt could not be canonically identified",
                )],
            };
        }
    };
    debug_assert!(request.portable_admission.admitted);
    debug_assert_eq!(request.receipt.target, request.expected_target);
    AotAdmissionResult {
        admission: Some(AotAdmission {
            schema: String::from(AOT_ADMISSION_SCHEMA),
            source_component_blake3: request.receipt.source_component_blake3,
            output: request.receipt.output,
            target: request.receipt.target,
            cpu_features: request.receipt.cpu_features,
            trust_class: String::from(AOT_TRUST_CLASS),
            wasmtime_configuration_blake3: request.receipt.wasmtime_configuration_blake3,
            cohort_blake3: request.receipt.cohort_blake3,
            wit_profile_blake3: request.receipt.wit_profile_blake3,
            build_inputs_blake3: request.receipt.build_inputs_blake3,
            receipt_blake3: receipt_identity,
        }),
        blockers: Vec::new(),
    }
}

fn validate_octet_binding(request: &PortableAdmissionRequest, blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    let Some(octet) = &request.octet_validation else {
        if request.octet_required {
            blockers.push(blocker(
                "missing-octet-report",
                "octet-report",
                "consumer/release portable admission requires an Octet artifact report",
            ));
        }
        debug_assert!(blockers.len() >= blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return;
    };
    if octet.artifact_blake3 != request.artifact.digest_blake3 {
        blockers.push(blocker(
            "octet-artifact-mismatch",
            "octet-report",
            "Octet report names different portable bytes",
        ));
    }
    if octet.profile_blake3 != request.expected_octet_profile_blake3
        || octet.cohort_blake3 != request.expected_octet_cohort_blake3
    {
        blockers.push(blocker(
            "octet-profile-mismatch",
            "octet-report",
            "Octet report profile or cohort identity differs from the declared rail",
        ));
    }
    if octet.decision != ValidationDecision::Pass {
        blockers.push(blocker(
            "octet-validation-failed",
            "octet-report",
            "Octet artifact rail did not admit the portable artifact",
        ));
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_transform_request(request: &TransformAdmissionRequest, blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    if is_count_above_bound(request.virtual_imports.len(), MAX_TRANSFORM_IMPORTS) {
        blockers.push(blocker("wizer-import-limit", "wizer", "Wizer virtual import set exceeds its fixed bound"));
    }
    if request.mode == WizerMode::Disabled {
        blockers.push(blocker("wizer-disabled", "wizer", "disabled Wizer configuration cannot admit transform output"));
    }
    if request.initialization_entrypoint.is_empty() {
        blockers.push(blocker(
            "missing-wizer-entrypoint",
            "wizer",
            "Wizer transform requires an explicit initialization entrypoint",
        ));
    }
    if request.input.size_bytes == 0 || request.first_output.size_bytes == 0 {
        blockers.push(blocker(
            "empty-wizer-artifact",
            "wizer",
            "Wizer input and output must be identified non-empty artifacts",
        ));
    }
    if is_count_above_bound(request.ambient_observations.len(), MAX_AMBIENT_OBSERVATIONS)
        || !request.ambient_observations.is_empty()
    {
        blockers.push(blocker(
            "wizer-ambient-state",
            "wizer",
            "Wizer initialization observed undeclared ambient state",
        ));
    }
    if !is_count_above_bound(request.virtual_imports.len(), MAX_TRANSFORM_IMPORTS) {
        blockers.reserve(request.virtual_imports.len());
        for import in &request.virtual_imports {
            if import.name.is_empty() || !import.deterministic || import.input_identity_blake3.is_none() {
                blockers.push(blocker(
                    "nondeterministic-wizer-import",
                    &import.name,
                    "Wizer imports must be deterministic and bind an identified virtual input",
                ));
            }
        }
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn repeated_output_matches(request: &TransformAdmissionRequest) -> bool {
    match &request.repeated_output {
        Some(repeated) => repeated.digest_blake3 == request.first_output.digest_blake3,
        None => request.mode != WizerMode::Deterministic,
    }
}

fn validate_aot_request(request: &AotAdmissionRequest, blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    if request.mode != AotMode::TrustedNative {
        blockers.push(blocker(
            "aot-disabled",
            "aot-receipt",
            "disabled AOT configuration cannot admit precompiled output",
        ));
    }
    if !request.portable_admission.admitted {
        blockers.push(blocker(
            "unvalidated-aot-source",
            "aot-receipt",
            "AOT source component must have passed portable admission",
        ));
    }
    if request.receipt.source_component_blake3 != request.portable_admission.artifact.digest_blake3 {
        blockers.push(blocker(
            "aot-source-mismatch",
            "aot-receipt",
            "AOT receipt names different portable source bytes",
        ));
    }
    if request.receipt.target != request.expected_target {
        blockers.push(blocker(
            "aot-target-mismatch",
            "aot-receipt",
            "precompiled artifact target differs from the expected release target",
        ));
    }
    if !aot_configuration_matches(request) {
        blockers.push(blocker(
            "aot-configuration-mismatch",
            "aot-receipt",
            "Wasmtime configuration, cohort, WIT profile, or build inputs differ from the expected release configuration",
        ));
    }
    if request.receipt.output.size_bytes == 0 || !request.receipt.output.logical_path.starts_with('/') {
        blockers.push(blocker(
            "invalid-aot-output",
            "aot-receipt",
            "precompiled output must be an identified non-empty store object",
        ));
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn aot_configuration_matches(request: &AotAdmissionRequest) -> bool {
    if request.receipt.wasmtime_configuration_blake3 != request.expected_wasmtime_configuration_blake3 {
        return false;
    }
    if request.receipt.cohort_blake3 != request.expected_cohort_blake3 {
        return false;
    }
    if request.receipt.wit_profile_blake3 != request.expected_wit_profile_blake3 {
        return false;
    }
    if request.receipt.build_inputs_blake3 != request.expected_build_inputs_blake3 {
        return false;
    }
    debug_assert_eq!(request.receipt.cohort_blake3, request.expected_cohort_blake3);
    debug_assert_eq!(request.receipt.build_inputs_blake3, request.expected_build_inputs_blake3);
    true
}

fn normalize_cpu_features(features: &mut Vec<String>, blockers: &mut Vec<ComponentBlocker>, subject: &str) {
    if is_count_above_bound(features.len(), MAX_CPU_FEATURES) {
        blockers.push(blocker("aot-cpu-feature-limit", subject, "AOT CPU feature set exceeds its fixed bound"));
        return;
    }
    if features.iter().any(|feature| feature.is_empty()) {
        blockers.push(blocker("empty-aot-cpu-feature", subject, "AOT CPU feature names must not be empty"));
    }
    let unique: BTreeSet<String> = features.iter().cloned().collect();
    if unique.len() != features.len() {
        blockers.push(blocker("duplicate-aot-cpu-feature", subject, "AOT CPU feature set contains duplicates"));
    }
    *features = unique.into_iter().collect();
}
