use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Serialize;

use crate::digest::BoundedValue;
use crate::digest::COHORT_REF_PREFIX;
use crate::digest::NamedValue;
use crate::digest::OBJECT_REF_PREFIX;
use crate::digest::PROFILE_REF_PREFIX;
use crate::digest::TypedRefValidation;
use crate::digest::digest_ref;
use crate::digest::validate_blake3;
use crate::digest::validate_git_revision;
use crate::digest::validate_identifier;
use crate::digest::validate_relative_path;
use crate::digest::validate_store_path;
use crate::digest::validate_typed_ref;
use crate::error::DigestError;
use crate::model::HARDWARE_PROFILE_SCHEMA;
use crate::model::HardwareBounds;
use crate::model::HardwareProfile;
use crate::model::HardwareTarget;
use crate::model::NIX_SEED_BOUNDARY;
use crate::model::ProfileValidation;
use crate::model::SYSTEM_X86_64_LINUX;
use crate::model::SourceClosureDecision;
use crate::model::SourceObservation;
use crate::model::SourcePackage;
use crate::model::TOOL_COHORT_SCHEMA;
use crate::model::ToolCohort;
use crate::model::ToolRole;

pub const REQUIRED_NON_CLAIMS: &[&str] = &[
    "no-commercial-simulator-or-license-server-support",
    "no-fpga-or-asic-correctness-proof",
    "no-physical-design-or-timing-closure-claim",
    "no-production-remote-farm-throughput-claim",
    "no-dvcon-speedup-reproduction-claim",
];

pub const HARD_MAX_SOURCE_PACKAGES: u32 = 32;
pub const HARD_MAX_SOURCE_FILES: u32 = 256;
pub const HARD_MAX_GENERATED_UNITS: u32 = 128;
pub const HARD_MAX_OPTIONS: u32 = 64;
pub const HARD_MAX_SMOKE_CASES: u32 = 32;
pub const HARD_MAX_OUTPUTS: u32 = 32;
pub const HARD_MAX_TEXT_BYTES: u32 = 4_096;
pub const HARD_MAX_PLAN_BYTES: u32 = 1_048_576;
pub const HARD_MAX_LOG_BYTES: u32 = 65_536;
pub const HARD_MAX_ACTIONS: u32 = 256;
pub const HARD_MAX_SOURCE_CLOSURE_STEPS: u32 = 256;

const PROFILE_DOMAIN: &[u8] = b"mantle.hardware.profile.v1";
const COHORT_DOMAIN: &[u8] = b"mantle.hardware.cohort.v1";
const REQUIRED_TOOL_ROLE_COUNT: usize = 5;
const SOURCE_OBSERVATION_DIAGNOSTIC_CAPACITY: usize = 32;

struct BoundValidation<'a> {
    value: u32,
    hard_maximum: u32,
    field: &'a str,
}

#[derive(Serialize)]
struct CohortHashable<'a> {
    schema: &'a str,
    seed_boundary: &'a str,
    system: &'a str,
    members: &'a [crate::model::ToolMember],
    closure_paths_blake3: &'a str,
}

pub fn cohort_ref(cohort: &ToolCohort) -> Result<String, DigestError> {
    let normalized = normalize_cohort(cohort.clone());
    let hashable = CohortHashable {
        schema: &normalized.schema,
        seed_boundary: &normalized.seed_boundary,
        system: &normalized.system,
        members: &normalized.members,
        closure_paths_blake3: &normalized.closure_paths_blake3,
    };
    let reference = digest_ref(COHORT_REF_PREFIX, COHORT_DOMAIN, &hashable)?;
    debug_assert!(reference.starts_with(COHORT_REF_PREFIX));
    debug_assert!(!normalized.members.is_empty());
    Ok(reference)
}

pub fn validate_profile(profile: HardwareProfile) -> Result<ProfileValidation, Vec<String>> {
    let normalized = normalize_profile(profile);
    let mut diagnostics = Vec::new();
    validate_profile_header(&normalized, &mut diagnostics);
    validate_bounds(&normalized.bounds, &mut diagnostics);
    validate_sources(&normalized, &mut diagnostics);
    validate_targets(&normalized, &mut diagnostics);
    validate_tool_cohort(&normalized.tool_cohort, &mut diagnostics);
    validate_options_and_cases(&normalized, &mut diagnostics);
    validate_non_claims(&normalized.non_claims, &mut diagnostics);
    let selected_source_ids =
        selected_source_closure(&normalized, &normalized.selected_target).unwrap_or_else(|errors| {
            diagnostics.extend(errors);
            Vec::new()
        });
    if !diagnostics.is_empty() {
        diagnostics.sort();
        diagnostics.dedup();
        return Err(diagnostics);
    }
    let selected_source_refs = source_refs(&normalized, &selected_source_ids);
    let cohort_ref = cohort_ref(&normalized.tool_cohort).map_err(|error| vec![String::from(error.code())])?;
    let profile_ref = digest_ref(PROFILE_REF_PREFIX, PROFILE_DOMAIN, &normalized)
        .map_err(|error| vec![String::from(error.code())])?;
    debug_assert!(!selected_source_ids.is_empty());
    debug_assert_eq!(selected_source_ids.len(), selected_source_refs.len());
    Ok(ProfileValidation {
        normalized,
        profile_ref,
        selected_source_ids,
        selected_source_refs,
        cohort_ref,
    })
}

pub fn selected_source_closure(profile: &HardwareProfile, target_id: &str) -> Result<Vec<String>, Vec<String>> {
    let packages = profile
        .source_packages
        .iter()
        .map(|package| (package.id.as_str(), package))
        .collect::<BTreeMap<_, _>>();
    let Some(target) = profile.targets.iter().find(|target| target.id == target_id) else {
        return Err(vec![String::from("selected-target-unknown")]);
    };
    let mut pending = target.source_packages.clone();
    let mut selected = BTreeSet::new();
    let mut steps = 0_u32;
    while let Some(package_id) = pending.pop() {
        steps = steps.saturating_add(1);
        if steps > HARD_MAX_SOURCE_CLOSURE_STEPS {
            return Err(vec![String::from("source-closure-step-limit-exceeded")]);
        }
        if !selected.insert(package_id.clone()) {
            continue;
        }
        let Some(package) = packages.get(package_id.as_str()) else {
            return Err(vec![String::from("undeclared-source-edge")]);
        };
        for dependency in &package.dependencies {
            pending.push(dependency.clone());
        }
    }
    if selected.is_empty() {
        return Err(vec![String::from("selected-source-closure-empty")]);
    }
    debug_assert!(steps <= HARD_MAX_SOURCE_CLOSURE_STEPS);
    debug_assert!(!selected.is_empty());
    Ok(selected.into_iter().collect())
}

pub fn validate_source_observations(
    profile: &HardwareProfile,
    target_id: &str,
    observations: Vec<SourceObservation>,
) -> SourceClosureDecision {
    let selected_source_ids = selected_source_closure(profile, target_id).unwrap_or_default();
    let selected = selected_source_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let declared = profile
        .source_packages
        .iter()
        .map(|package| (package.id.as_str(), package))
        .collect::<BTreeMap<_, _>>();
    let mut observed = BTreeMap::new();
    let mut rejected = Vec::with_capacity(SOURCE_OBSERVATION_DIAGNOSTIC_CAPACITY);
    for observation in observations {
        if observed.insert(observation.id.clone(), observation).is_some() {
            rejected.push(String::from("duplicate-source-observation"));
        }
    }
    for source_id in &selected_source_ids {
        match (declared.get(source_id.as_str()), observed.get(source_id)) {
            (Some(package), Some(observation)) => validate_observation(package, observation, &mut rejected),
            (Some(_), None) => rejected.push(String::from("selected-source-observation-missing")),
            (None, _) => rejected.push(String::from("undeclared-source-edge")),
        }
    }
    for (source_id, observation) in &observed {
        if !selected.contains(source_id.as_str()) && observation.acquired {
            rejected.push(String::from("unrelated-source-acquired"));
        }
    }
    rejected.sort();
    rejected.dedup();
    let selected_source_refs = source_refs(profile, &selected_source_ids);
    debug_assert_eq!(selected_source_ids.len(), selected_source_refs.len());
    debug_assert!(rejected.iter().all(|diagnostic| !diagnostic.is_empty()));
    SourceClosureDecision {
        selected_source_ids,
        selected_source_refs,
        rejected,
    }
}

fn validate_observation(package: &SourcePackage, observation: &SourceObservation, rejected: &mut Vec<String>) {
    if !observation.acquired {
        rejected.push(String::from("selected-source-not-acquired"));
    }
    if package.locator != observation.locator {
        rejected.push(String::from("source-locator-drift"));
    }
    if package.revision != observation.revision {
        rejected.push(String::from("source-revision-drift"));
    }
    if package.recursive_blake3 != observation.recursive_blake3 {
        rejected.push(String::from("source-recursive-digest-drift"));
    }
    if package.sentinel_blake3 != observation.sentinel_blake3 {
        rejected.push(String::from("source-sentinel-drift"));
    }
    let mut expected_edges = package.dependencies.clone();
    let mut actual_edges = observation.declared_edges.clone();
    expected_edges.sort();
    actual_edges.sort();
    if expected_edges != actual_edges {
        rejected.push(String::from("undeclared-source-edge"));
    }
    debug_assert!(!package.id.is_empty());
    debug_assert!(!observation.id.is_empty());
}

fn normalize_profile(mut profile: HardwareProfile) -> HardwareProfile {
    profile.source_packages = profile.source_packages.into_iter().map(normalize_source).collect();
    profile.source_packages.sort_by(|left, right| left.id.cmp(&right.id));
    profile.targets = profile.targets.into_iter().map(normalize_target).collect();
    profile.targets.sort_by(|left, right| left.id.cmp(&right.id));
    profile.tool_cohort = normalize_cohort(profile.tool_cohort);
    profile.smoke_cases.sort_by(|left, right| left.id.cmp(&right.id));
    profile.expected_outputs.sort();
    profile.non_claims.sort();
    profile
}

fn normalize_source(mut source: SourcePackage) -> SourcePackage {
    source.dependencies.sort();
    source.files.sort();
    source
}

fn normalize_target(mut target: HardwareTarget) -> HardwareTarget {
    target.source_packages.sort();
    target.source_files.sort();
    target
}

fn normalize_cohort(mut cohort: ToolCohort) -> ToolCohort {
    cohort.members.sort();
    cohort
}

fn validate_profile_header(profile: &HardwareProfile, diagnostics: &mut Vec<String>) {
    let diagnostics_before = diagnostics.len();
    if profile.schema != HARDWARE_PROFILE_SCHEMA {
        diagnostics.push(String::from("hardware-profile-schema-unsupported"));
    }
    push_result(
        diagnostics,
        validate_identifier(BoundedValue {
            value: &profile.profile_id,
            field: "profile-id",
            maximum_bytes: profile.bounds.max_text_bytes,
        }),
    );
    push_result(
        diagnostics,
        validate_identifier(BoundedValue {
            value: &profile.selected_target,
            field: "selected-target",
            maximum_bytes: profile.bounds.max_text_bytes,
        }),
    );
    if profile.support_tier != "heavy-capability-gated" {
        diagnostics.push(String::from("hardware-support-tier-invalid"));
    }
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics[diagnostics_before..].iter().all(|diagnostic| !diagnostic.is_empty()));
}

fn validate_bounds(bounds: &HardwareBounds, diagnostics: &mut Vec<String>) {
    for input in [
        BoundValidation {
            value: bounds.max_source_packages,
            hard_maximum: HARD_MAX_SOURCE_PACKAGES,
            field: "source-packages",
        },
        BoundValidation {
            value: bounds.max_source_files,
            hard_maximum: HARD_MAX_SOURCE_FILES,
            field: "source-files",
        },
        BoundValidation {
            value: bounds.max_generated_units,
            hard_maximum: HARD_MAX_GENERATED_UNITS,
            field: "generated-units",
        },
        BoundValidation {
            value: bounds.max_options,
            hard_maximum: HARD_MAX_OPTIONS,
            field: "options",
        },
        BoundValidation {
            value: bounds.max_smoke_cases,
            hard_maximum: HARD_MAX_SMOKE_CASES,
            field: "smoke-cases",
        },
        BoundValidation {
            value: bounds.max_outputs,
            hard_maximum: HARD_MAX_OUTPUTS,
            field: "outputs",
        },
        BoundValidation {
            value: bounds.max_text_bytes,
            hard_maximum: HARD_MAX_TEXT_BYTES,
            field: "text-bytes",
        },
        BoundValidation {
            value: bounds.max_plan_bytes,
            hard_maximum: HARD_MAX_PLAN_BYTES,
            field: "plan-bytes",
        },
        BoundValidation {
            value: bounds.max_log_bytes,
            hard_maximum: HARD_MAX_LOG_BYTES,
            field: "log-bytes",
        },
        BoundValidation {
            value: bounds.max_actions,
            hard_maximum: HARD_MAX_ACTIONS,
            field: "actions",
        },
    ] {
        validate_bound(input, diagnostics);
    }
}

fn validate_bound(input: BoundValidation<'_>, diagnostics: &mut Vec<String>) {
    if input.value == 0 || input.value > input.hard_maximum {
        diagnostics.push(alloc::format!("{}-bound-invalid", input.field));
    }
}

fn validate_sources(profile: &HardwareProfile, diagnostics: &mut Vec<String>) {
    let diagnostics_before = diagnostics.len();
    validate_count(profile.source_packages.len(), profile.bounds.max_source_packages, "source-package", diagnostics);
    let mut ids = BTreeSet::new();
    let mut total_file_count = 0_u32;
    let mut is_total_file_count_representable = true;
    for package in &profile.source_packages {
        if !ids.insert(package.id.as_str()) {
            diagnostics.push(String::from("duplicate-source-package"));
        }
        validate_source_package(package, &profile.bounds, diagnostics);
        match u32::try_from(package.files.len()) {
            Ok(package_file_count) => {
                total_file_count = total_file_count.saturating_add(package_file_count);
            }
            Err(_) => {
                is_total_file_count_representable = false;
            }
        }
    }
    let is_total_file_count_valid = is_total_file_count_representable
        && total_file_count > 0
        && total_file_count <= profile.bounds.max_source_files;
    if !is_total_file_count_valid {
        diagnostics.push(String::from("source-file-bound-exceeded"));
    }
    debug_assert!(ids.len() <= profile.source_packages.len());
    debug_assert!(diagnostics.len() >= diagnostics_before);
}

fn validate_source_package(package: &SourcePackage, bounds: &HardwareBounds, diagnostics: &mut Vec<String>) {
    let diagnostics_before = diagnostics.len();
    push_result(
        diagnostics,
        validate_identifier(BoundedValue {
            value: &package.id,
            field: "source-id",
            maximum_bytes: bounds.max_text_bytes,
        }),
    );
    if !package.locator.starts_with("fixture-git://") {
        diagnostics.push(String::from("source-locator-not-pinned-local-git"));
    }
    push_result(diagnostics, validate_git_revision(&package.revision));
    push_result(
        diagnostics,
        validate_blake3(NamedValue {
            value: &package.recursive_blake3,
            field: "source-recursive",
        }),
    );
    push_result(
        diagnostics,
        validate_blake3(NamedValue {
            value: &package.sentinel_blake3,
            field: "source-sentinel",
        }),
    );
    push_result(
        diagnostics,
        validate_typed_ref(TypedRefValidation {
            value: &package.object_ref,
            prefix: OBJECT_REF_PREFIX,
            field: "source-object-ref",
        }),
    );
    if package.files.is_empty() {
        diagnostics.push(String::from("source-files-empty"));
    }
    for file in &package.files {
        push_result(
            diagnostics,
            validate_relative_path(BoundedValue {
                value: file,
                field: "source-file",
                maximum_bytes: bounds.max_text_bytes,
            }),
        );
    }
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics[diagnostics_before..].iter().all(|diagnostic| !diagnostic.is_empty()));
}

fn validate_targets(profile: &HardwareProfile, diagnostics: &mut Vec<String>) {
    if profile.targets.is_empty() {
        diagnostics.push(String::from("hardware-targets-empty"));
    }
    let mut ids = BTreeSet::new();
    for target in &profile.targets {
        if !ids.insert(target.id.as_str()) {
            diagnostics.push(String::from("duplicate-hardware-target"));
        }
        validate_target(target, &profile.bounds, diagnostics);
    }
    if !profile.targets.iter().any(|target| target.id == profile.selected_target) {
        diagnostics.push(String::from("selected-target-unknown"));
    }
}

fn validate_target(target: &HardwareTarget, bounds: &HardwareBounds, diagnostics: &mut Vec<String>) {
    let diagnostics_before = diagnostics.len();
    push_result(
        diagnostics,
        validate_identifier(BoundedValue {
            value: &target.id,
            field: "target-id",
            maximum_bytes: bounds.max_text_bytes,
        }),
    );
    push_result(
        diagnostics,
        validate_identifier(BoundedValue {
            value: &target.top_module,
            field: "top-module",
            maximum_bytes: bounds.max_text_bytes,
        }),
    );
    if target.source_packages.is_empty() || target.source_files.is_empty() {
        diagnostics.push(String::from("target-source-set-empty"));
    }
    for file in &target.source_files {
        push_result(
            diagnostics,
            validate_relative_path(BoundedValue {
                value: file,
                field: "target-source-file",
                maximum_bytes: bounds.max_text_bytes,
            }),
        );
    }
    push_result(
        diagnostics,
        validate_relative_path(BoundedValue {
            value: &target.reference_model,
            field: "reference-model",
            maximum_bytes: bounds.max_text_bytes,
        }),
    );
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics[diagnostics_before..].iter().all(|diagnostic| !diagnostic.is_empty()));
}

fn validate_tool_cohort(cohort: &ToolCohort, diagnostics: &mut Vec<String>) {
    let diagnostics_before = diagnostics.len();
    if cohort.schema != TOOL_COHORT_SCHEMA {
        diagnostics.push(String::from("tool-cohort-schema-unsupported"));
    }
    if cohort.seed_boundary != NIX_SEED_BOUNDARY {
        diagnostics.push(String::from("tool-cohort-seed-boundary-invalid"));
    }
    if cohort.system != SYSTEM_X86_64_LINUX {
        diagnostics.push(String::from("tool-cohort-system-unsupported"));
    }
    push_result(
        diagnostics,
        validate_blake3(NamedValue {
            value: &cohort.identity_blake3,
            field: "tool-cohort-identity",
        }),
    );
    push_result(
        diagnostics,
        validate_blake3(NamedValue {
            value: &cohort.closure_paths_blake3,
            field: "tool-closure-paths",
        }),
    );
    let expected_identity = cohort_ref(cohort)
        .ok()
        .and_then(|reference| reference.strip_prefix(COHORT_REF_PREFIX).map(String::from));
    if expected_identity.as_deref() != Some(cohort.identity_blake3.as_str()) {
        diagnostics.push(String::from("tool-cohort-identity-mismatch"));
    }
    let roles = cohort.members.iter().map(|member| member.role).collect::<BTreeSet<_>>();
    if roles.len() != REQUIRED_TOOL_ROLE_COUNT || cohort.members.len() != REQUIRED_TOOL_ROLE_COUNT {
        diagnostics.push(String::from("tool-cohort-role-set-invalid"));
    }
    for member in &cohort.members {
        validate_tool_member(member, diagnostics);
    }
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics[diagnostics_before..].iter().all(|diagnostic| !diagnostic.is_empty()));
}

fn validate_tool_member(member: &crate::model::ToolMember, diagnostics: &mut Vec<String>) {
    let diagnostics_before = diagnostics.len();
    if member.package.is_empty() || member.version.is_empty() {
        diagnostics.push(String::from("tool-member-identity-empty"));
    }
    push_result(
        diagnostics,
        validate_store_path(NamedValue {
            value: &member.store_path,
            field: "tool-store-path",
        }),
    );
    push_result(
        diagnostics,
        validate_relative_path(BoundedValue {
            value: &member.executable,
            field: "tool-executable",
            maximum_bytes: HARD_MAX_TEXT_BYTES,
        }),
    );
    push_result(
        diagnostics,
        validate_blake3(NamedValue {
            value: &member.binary_blake3,
            field: "tool-binary",
        }),
    );
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics[diagnostics_before..].iter().all(|diagnostic| !diagnostic.is_empty()));
}

fn validate_options_and_cases(profile: &HardwareProfile, diagnostics: &mut Vec<String>) {
    let diagnostics_before = diagnostics.len();
    if profile.generation.language != "systemverilog" || profile.generation.output_prefix.is_empty() {
        diagnostics.push(String::from("generation-options-invalid"));
    }
    validate_count(profile.generation.extra_args.len(), profile.bounds.max_options, "generation-option", diagnostics);
    validate_count(profile.compile_flags.len(), profile.bounds.max_options, "compile-option", diagnostics);
    validate_count(profile.link_flags.len(), profile.bounds.max_options, "link-option", diagnostics);
    validate_count(profile.smoke_cases.len(), profile.bounds.max_smoke_cases, "smoke-case", diagnostics);
    validate_count(profile.expected_outputs.len(), profile.bounds.max_outputs, "expected-output", diagnostics);
    let mut case_ids = BTreeSet::new();
    for case in &profile.smoke_cases {
        if !case_ids.insert(case.id.as_str()) {
            diagnostics.push(String::from("duplicate-smoke-case"));
        }
        push_result(
            diagnostics,
            validate_identifier(BoundedValue {
                value: &case.id,
                field: "smoke-case-id",
                maximum_bytes: profile.bounds.max_text_bytes,
            }),
        );
        if case.input.is_empty() || case.expected.is_empty() || case.max_log_bytes == 0 {
            diagnostics.push(String::from("smoke-case-invalid"));
        }
        if case.max_log_bytes > profile.bounds.max_log_bytes {
            diagnostics.push(String::from("smoke-log-bound-exceeded"));
        }
    }
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics[diagnostics_before..].iter().all(|diagnostic| !diagnostic.is_empty()));
}

fn validate_non_claims(non_claims: &[String], diagnostics: &mut Vec<String>) {
    let declared = non_claims.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for required in REQUIRED_NON_CLAIMS {
        if !declared.contains(required) {
            diagnostics.push(alloc::format!("required-non-claim-missing:{required}"));
        }
    }
}

fn validate_count(actual: usize, maximum: u32, field: &str, diagnostics: &mut Vec<String>) {
    let is_count_valid = u32::try_from(actual).is_ok_and(|actual_count| actual_count > 0 && actual_count <= maximum);
    if !is_count_valid {
        diagnostics.push(alloc::format!("{field}-count-invalid"));
    }
}

fn source_refs(profile: &HardwareProfile, selected_ids: &[String]) -> Vec<String> {
    let by_id = profile
        .source_packages
        .iter()
        .map(|package| (package.id.as_str(), &package.object_ref))
        .collect::<BTreeMap<_, _>>();
    selected_ids
        .iter()
        .filter_map(|id| by_id.get(id.as_str()).map(|reference| (*reference).clone()))
        .collect()
}

fn push_result<E: core::fmt::Display>(diagnostics: &mut Vec<String>, result: Result<(), E>) {
    if let Err(error) = result {
        diagnostics.push(alloc::format!("{error}"));
    }
}

pub fn required_tool_roles() -> BTreeSet<ToolRole> {
    [
        ToolRole::Verilator,
        ToolRole::CxxCompiler,
        ToolRole::Linker,
        ToolRole::RuntimeSupport,
        ToolRole::Shell,
    ]
    .into_iter()
    .collect()
}
