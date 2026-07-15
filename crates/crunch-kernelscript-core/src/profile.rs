use alloc::collections::BTreeSet;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Serialize;

use crate::ArtifactIdentity;
use crate::ArtifactOutputClass;
use crate::Blake3Digest;
use crate::CompilerDependencyLock;
use crate::ExperimentBlocker;
use crate::ExperimentBounds;
use crate::ExperimentProfile;
use crate::GeneratedFileClass;
use crate::KernelTarget;
use crate::TargetInputIdentity;
use crate::TargetInputRole;
use crate::ToolIdentity;
use crate::ToolRole;
use crate::blocker::blocker;
use crate::digest::canonical_identity;
use crate::digest::count_above_bound;

pub const EXPERIMENT_PROFILE_SCHEMA: &str = "mantle-kernelscript-experiment-v1";
pub const REQUIRED_NON_CLAIMS: &[&str] = &[
    "beta-language-soundness-not-proven",
    "bpf-verifier-acceptance-not-proven",
    "kernel-safety-not-proven",
    "runtime-correctness-not-proven",
    "deployability-not-proven",
    "production-readiness-not-proven",
    "onix-semantics-external",
    "chaoscontrol-runtime-evidence-required",
];

const MAX_GENERATED_FILES_LIMIT: u32 = 64;
const MAX_GENERATED_FILE_BYTES_LIMIT: u64 = 8_388_608;
const MAX_GENERATED_TOTAL_BYTES_LIMIT: u64 = 33_554_432;
const MAX_COMPILATION_STEPS_LIMIT: u32 = 32;
const MAX_OUTPUT_FILES_LIMIT: u32 = 16;
const MAX_OUTPUT_BYTES_LIMIT: u64 = 67_108_864;
const MAX_ELF_SECTIONS_LIMIT: u32 = 512;
const MAX_SECTION_NAME_BYTES_LIMIT: u32 = 256;
const MAX_RECEIPT_BLOCKERS_LIMIT: u32 = 128;
const MAX_TEXT_BYTES_LIMIT: u32 = 4_096;
const MAX_TOOLCHAIN_MEMBERS: u32 = 16;
const MAX_COMPILER_DEPENDENCIES: u32 = 256;
const MAX_OUTPUT_CLASSES: u32 = 8;
const MAX_NON_CLAIMS: u32 = 32;
const MAX_FLAGS_PER_CLASS: u32 = 32;
const GIT_REVISION_HEX_LENGTH: usize = 40;
const ARTIFACT_REF_PREFIX: &str = "mantle://blake3/";
pub const ONIX_KERNEL_BUILD_IDENTITY_PREFIX: &str = "onix:blake3:kernel-build:";
pub const OBSERVATION_KERNEL_BUILD_IDENTITY_PREFIX: &str = "observation:blake3:kernel-build:";
pub const OPAM_LOCK_FORMAT: &str = "opam-lock";
pub const NIX_CLOSURE_OBSERVATION_LOCK_FORMAT: &str = "nix-closure-observation";
const SOURCE_EXTENSION: &str = ".ks";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileValidation {
    pub profile_identity_blake3: Option<Blake3Digest>,
    pub blockers: Vec<ExperimentBlocker>,
}

struct TextValidation<'a> {
    value: &'a str,
    subject: &'a str,
    maximum_bytes: u32,
}

struct BoundU32Validation<'a> {
    value: u32,
    maximum: u32,
    subject: &'a str,
}

struct BoundU64Validation<'a> {
    value: u64,
    maximum: u64,
    subject: &'a str,
}

pub fn validate_profile(profile: ExperimentProfile) -> ProfileValidation {
    let mut blockers = Vec::new();
    validate_header(&profile, &mut blockers);
    validate_source_and_compiler(&profile, &mut blockers);
    validate_bounds(&profile.bounds, &mut blockers);
    validate_toolchain(&profile, &mut blockers);
    validate_target(&profile.target, &mut blockers);
    validate_outputs(&profile, &mut blockers);
    validate_flags(&profile, &mut blockers);
    validate_non_claims(&profile, &mut blockers);
    let profile_identity_blake3 = identity_if_clean(&profile, &mut blockers);
    debug_assert!(blockers.is_empty() == profile_identity_blake3.is_some());
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
    ProfileValidation {
        profile_identity_blake3,
        blockers,
    }
}

fn validate_header(profile: &ExperimentProfile, blockers: &mut Vec<ExperimentBlocker>) {
    let initial_blocker_count: usize = blockers.len();
    if profile.schema != EXPERIMENT_PROFILE_SCHEMA {
        blockers.push(blocker(
            "unsupported-profile-schema",
            "profile.schema",
            "KernelScript profile schema is unsupported",
        ));
    }
    validate_text(
        TextValidation {
            value: &profile.experiment_id,
            subject: "profile.experiment_id",
            maximum_bytes: profile.bounds.max_text_bytes,
        },
        blockers,
    );
    if !profile.beta || profile.enabled_by_default {
        blockers.push(blocker(
            "experiment-not-bounded-beta",
            "profile",
            "KernelScript must remain beta and disabled by default",
        ));
    }
    debug_assert!(!EXPERIMENT_PROFILE_SCHEMA.is_empty());
    debug_assert!(blockers.len() >= initial_blocker_count);
}

fn validate_source_and_compiler(profile: &ExperimentProfile, blockers: &mut Vec<ExperimentBlocker>) {
    let initial_blocker_count: usize = blockers.len();
    let has_safe_source_path = safe_relative_path(&profile.source.relative_path);
    let has_source_extension = profile.source.relative_path.ends_with(SOURCE_EXTENSION);
    if !has_safe_source_path || !has_source_extension {
        blockers.push(blocker(
            "invalid-kernelscript-source-path",
            "profile.source",
            "KernelScript source must be one safe relative .ks path",
        ));
    }
    if profile.source.size_bytes == 0 {
        blockers.push(blocker(
            "empty-kernelscript-source",
            "profile.source",
            "KernelScript source must have positive size",
        ));
    }
    let compiler = &profile.compiler;
    validate_text(
        TextValidation {
            value: &compiler.version,
            subject: "compiler.version",
            maximum_bytes: profile.bounds.max_text_bytes,
        },
        blockers,
    );
    if !lower_hex(&compiler.source_revision, GIT_REVISION_HEX_LENGTH) {
        blockers.push(blocker(
            "invalid-compiler-revision",
            "compiler.source_revision",
            "compiler revision must be exact Git hex",
        ));
    }
    let is_https_source = compiler.source_archive_url.starts_with("https://");
    let has_embedded_credentials = compiler.source_archive_url.contains('@');
    if !is_https_source || has_embedded_credentials {
        blockers.push(blocker(
            "invalid-compiler-source-url",
            "compiler.source_archive_url",
            "compiler source URL must be credential-free HTTPS",
        ));
    }
    validate_dependency_lock(&compiler.dependency_lock, profile.bounds.max_text_bytes, blockers);
    validate_artifact(&compiler.compiler_executable, "compiler.executable", blockers);
    debug_assert!(GIT_REVISION_HEX_LENGTH > 0);
    debug_assert!(blockers.len() >= initial_blocker_count);
}

fn validate_dependency_lock(lock: &CompilerDependencyLock, text_bound: u32, blockers: &mut Vec<ExperimentBlocker>) {
    let initial_blocker_count: usize = blockers.len();
    if !has_valid_dependency_lock_shape(lock) {
        blockers.push(blocker(
            "incomplete-compiler-dependency-lock",
            "compiler.dependency_lock",
            "compiler dependencies require an exact bounded non-empty opam-lock or Nix closure observation cohort",
        ));
        return;
    }
    let mut packages = BTreeSet::new();
    let mut artifacts = BTreeSet::new();
    for dependency in &lock.dependencies {
        validate_text(
            TextValidation {
                value: &dependency.package,
                subject: "compiler.dependency.package",
                maximum_bytes: text_bound,
            },
            blockers,
        );
        validate_text(
            TextValidation {
                value: &dependency.version,
                subject: "compiler.dependency.version",
                maximum_bytes: text_bound,
            },
            blockers,
        );
        validate_artifact(&dependency.artifact, "compiler.dependency.artifact", blockers);
        if !packages.insert(dependency.package.as_str()) {
            blockers.push(blocker(
                "duplicate-compiler-dependency-package",
                &dependency.package,
                "compiler dependency package occurs more than once",
            ));
        }
        if !artifacts.insert(&dependency.artifact) {
            blockers.push(blocker(
                "duplicate-compiler-dependency-artifact",
                &dependency.package,
                "compiler dependency artifact identity occurs more than once",
            ));
        }
    }
    debug_assert!(packages.len() <= lock.dependencies.len());
    debug_assert!(blockers.len() >= initial_blocker_count);
}

fn has_valid_dependency_lock_shape(lock: &CompilerDependencyLock) -> bool {
    let has_recognized_lock_kind =
        matches!(lock.format.as_str(), OPAM_LOCK_FORMAT | NIX_CLOSURE_OBSERVATION_LOCK_FORMAT);
    let has_dependencies = !lock.dependencies.is_empty();
    let is_dependency_count_bounded = !count_above_bound(lock.dependencies.len(), MAX_COMPILER_DEPENDENCIES);
    let is_count_consistent = usize::try_from(lock.dependency_count).ok() == Some(lock.dependencies.len());
    has_recognized_lock_kind && has_dependencies && is_dependency_count_bounded && is_count_consistent
}

fn validate_bounds(bounds: &ExperimentBounds, blockers: &mut Vec<ExperimentBlocker>) {
    debug_assert!(MAX_GENERATED_FILES_LIMIT > 0);
    debug_assert!(MAX_GENERATED_FILE_BYTES_LIMIT > 0);
    validate_u32_bounds(bounds, blockers);
    validate_u64_bounds(bounds, blockers);
}

fn validate_u32_bounds(bounds: &ExperimentBounds, blockers: &mut Vec<ExperimentBlocker>) {
    debug_assert!(MAX_COMPILATION_STEPS_LIMIT > 0);
    debug_assert!(MAX_RECEIPT_BLOCKERS_LIMIT > 0);
    for validation in [
        BoundU32Validation {
            value: bounds.max_generated_files,
            maximum: MAX_GENERATED_FILES_LIMIT,
            subject: "max_generated_files",
        },
        BoundU32Validation {
            value: bounds.max_compilation_steps,
            maximum: MAX_COMPILATION_STEPS_LIMIT,
            subject: "max_compilation_steps",
        },
        BoundU32Validation {
            value: bounds.max_output_files,
            maximum: MAX_OUTPUT_FILES_LIMIT,
            subject: "max_output_files",
        },
        BoundU32Validation {
            value: bounds.max_elf_sections,
            maximum: MAX_ELF_SECTIONS_LIMIT,
            subject: "max_elf_sections",
        },
        BoundU32Validation {
            value: bounds.max_section_name_bytes,
            maximum: MAX_SECTION_NAME_BYTES_LIMIT,
            subject: "max_section_name_bytes",
        },
        BoundU32Validation {
            value: bounds.max_receipt_blockers,
            maximum: MAX_RECEIPT_BLOCKERS_LIMIT,
            subject: "max_receipt_blockers",
        },
        BoundU32Validation {
            value: bounds.max_text_bytes,
            maximum: MAX_TEXT_BYTES_LIMIT,
            subject: "max_text_bytes",
        },
    ] {
        validate_bound_u32(validation, blockers);
    }
}

fn validate_u64_bounds(bounds: &ExperimentBounds, blockers: &mut Vec<ExperimentBlocker>) {
    debug_assert!(MAX_GENERATED_TOTAL_BYTES_LIMIT > 0);
    debug_assert!(MAX_OUTPUT_BYTES_LIMIT > 0);
    for validation in [
        BoundU64Validation {
            value: bounds.max_generated_file_bytes,
            maximum: MAX_GENERATED_FILE_BYTES_LIMIT,
            subject: "max_generated_file_bytes",
        },
        BoundU64Validation {
            value: bounds.max_generated_total_bytes,
            maximum: MAX_GENERATED_TOTAL_BYTES_LIMIT,
            subject: "max_generated_total_bytes",
        },
        BoundU64Validation {
            value: bounds.max_output_bytes,
            maximum: MAX_OUTPUT_BYTES_LIMIT,
            subject: "max_output_bytes",
        },
    ] {
        validate_bound_u64(validation, blockers);
    }
}

fn validate_toolchain(profile: &ExperimentProfile, blockers: &mut Vec<ExperimentBlocker>) {
    let initial_blocker_count: usize = blockers.len();
    if count_above_bound(profile.toolchain.len(), MAX_TOOLCHAIN_MEMBERS) {
        blockers.push(blocker(
            "toolchain-member-limit",
            "profile.toolchain",
            "toolchain exceeds the fixed member bound",
        ));
        return;
    }
    let mut roles = BTreeSet::new();
    for tool in &profile.toolchain {
        validate_tool(tool, profile.bounds.max_text_bytes, blockers);
        if !roles.insert(tool.role) {
            blockers.push(blocker("duplicate-tool-role", "profile.toolchain", "toolchain role is duplicated"));
        }
    }
    for required in required_tool_roles(profile) {
        if !roles.contains(&required) {
            blockers.push(blocker("missing-tool-role", tool_role_name(required), "required toolchain role is absent"));
        }
    }
    debug_assert!(roles.len() <= profile.toolchain.len());
    debug_assert!(blockers.len() >= initial_blocker_count);
}

fn validate_tool(tool: &ToolIdentity, text_bound: u32, blockers: &mut Vec<ExperimentBlocker>) {
    validate_text(
        TextValidation {
            value: &tool.version,
            subject: "tool.version",
            maximum_bytes: text_bound,
        },
        blockers,
    );
    validate_artifact(&tool.executable_or_library, "tool.artifact", blockers);
}

fn required_tool_roles(profile: &ExperimentProfile) -> Vec<ToolRole> {
    let mut roles = vec![
        ToolRole::Ocaml,
        ToolRole::Dune,
        ToolRole::Menhir,
        ToolRole::Clang,
        ToolRole::CCompiler,
        ToolRole::Bpftool,
        ToolRole::Libbpf,
        ToolRole::ElfLibrary,
        ToolRole::Zlib,
    ];
    if profile.output_classes.contains(&ArtifactOutputClass::KernelModule) {
        roles.push(ToolRole::KernelBuild);
    }
    roles
}

fn validate_target(target: &KernelTarget, blockers: &mut Vec<ExperimentBlocker>) {
    if !valid_kernel_build_identity(&target.kernel_build_identity) || target.kernel_release.is_empty() {
        blockers.push(blocker(
            "invalid-kernel-target",
            "profile.target",
            "target kernel identity and release must be explicit",
        ));
    }
    for (expected_role, input) in [
        (TargetInputRole::Btf, &target.btf),
        (TargetInputRole::Headers, &target.headers),
        (TargetInputRole::Config, &target.config),
    ] {
        validate_target_input(target, expected_role, input, blockers);
    }
}

fn validate_target_input(
    target: &KernelTarget,
    expected_role: TargetInputRole,
    input: &TargetInputIdentity,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    let initial_blocker_count: usize = blockers.len();
    validate_artifact(&input.artifact, "target.input", blockers);
    if input.role != expected_role {
        blockers.push(blocker(
            "kernel-input-role-mismatch",
            "target.input",
            "kernel input occupies the wrong typed role",
        ));
    }
    let has_target_architecture = input.architecture == target.architecture;
    let has_target_release = input.kernel_release == target.kernel_release;
    let has_target_identity = input.kernel_build_identity == target.kernel_build_identity;
    if !has_target_architecture || !has_target_release || !has_target_identity {
        blockers.push(blocker(
            "kernel-input-cohort-mismatch",
            "target.input",
            "kernel input does not bind the exact target cohort",
        ));
    }
    debug_assert!(blockers.len() >= initial_blocker_count);
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
}

fn validate_outputs(profile: &ExperimentProfile, blockers: &mut Vec<ExperimentBlocker>) {
    if profile.output_classes.is_empty() || count_above_bound(profile.output_classes.len(), MAX_OUTPUT_CLASSES) {
        blockers.push(blocker(
            "output-class-limit",
            "profile.output_classes",
            "output classes must be bounded and non-empty",
        ));
        return;
    }
    let unique_outputs: BTreeSet<ArtifactOutputClass> = profile.output_classes.iter().copied().collect();
    if unique_outputs.len() != profile.output_classes.len() {
        blockers.push(blocker("duplicate-output-class", "profile.output_classes", "output class is duplicated"));
    }
    validate_expected_files(profile, blockers);
    require_output_file_classes(profile, blockers);
}

fn validate_expected_files(profile: &ExperimentProfile, blockers: &mut Vec<ExperimentBlocker>) {
    let initial_blocker_count: usize = blockers.len();
    if profile.expected_generated_files.is_empty()
        || count_above_bound(profile.expected_generated_files.len(), profile.bounds.max_generated_files)
    {
        blockers.push(blocker(
            "expected-file-limit",
            "profile.expected_generated_files",
            "expected generated files exceed the profile bound",
        ));
        return;
    }
    let mut paths = BTreeSet::new();
    for expected in &profile.expected_generated_files {
        let has_safe_path = safe_relative_path(&expected.relative_path);
        let has_positive_bound = expected.max_bytes > 0;
        let is_bound_admitted = expected.max_bytes <= profile.bounds.max_generated_file_bytes;
        if !has_safe_path || !has_positive_bound || !is_bound_admitted {
            blockers.push(blocker(
                "invalid-expected-file",
                &expected.relative_path,
                "expected file path or byte bound is invalid",
            ));
        }
        if !paths.insert(expected.relative_path.clone()) {
            blockers.push(blocker(
                "duplicate-expected-file",
                &expected.relative_path,
                "expected generated file path is duplicated",
            ));
        }
    }
    debug_assert!(paths.len() <= profile.expected_generated_files.len());
    debug_assert!(blockers.len() >= initial_blocker_count);
}

fn require_output_file_classes(profile: &ExperimentProfile, blockers: &mut Vec<ExperimentBlocker>) {
    let classes: BTreeSet<GeneratedFileClass> =
        profile.expected_generated_files.iter().map(|file| file.class).collect();
    require_generated_class(&classes, GeneratedFileClass::MakefileEvidence, "missing-makefile-evidence", blockers);
    require_generated_class(&classes, GeneratedFileClass::EbpfC, "missing-ebpf-source", blockers);
    if profile.output_classes.contains(&ArtifactOutputClass::UserspaceLoader) {
        require_generated_class(&classes, GeneratedFileClass::UserspaceC, "missing-userspace-source", blockers);
    }
    if profile.output_classes.contains(&ArtifactOutputClass::KernelModule) {
        require_generated_class(&classes, GeneratedFileClass::ModuleC, "missing-module-source", blockers);
        require_generated_class(&classes, GeneratedFileClass::KbuildEvidence, "missing-kbuild-evidence", blockers);
    }
    if profile.output_classes.contains(&ArtifactOutputClass::TestBinary) {
        require_generated_class(&classes, GeneratedFileClass::TestC, "missing-test-source", blockers);
    }
}

fn require_generated_class(
    classes: &BTreeSet<GeneratedFileClass>,
    required: GeneratedFileClass,
    code: &str,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    if !classes.contains(&required) {
        blockers.push(blocker(
            code,
            "profile.expected_generated_files",
            "selected output lacks its exact generated source class",
        ));
    }
}

fn validate_flags(profile: &ExperimentProfile, blockers: &mut Vec<ExperimentBlocker>) {
    for (subject, flags) in [
        ("profile.bpf_compiler_flags", &profile.bpf_compiler_flags),
        ("profile.userspace_compiler_flags", &profile.userspace_compiler_flags),
        ("profile.module_compiler_flags", &profile.module_compiler_flags),
    ] {
        if count_above_bound(flags.len(), MAX_FLAGS_PER_CLASS) {
            blockers.push(blocker("compiler-flag-limit", subject, "compiler flag list exceeds the fixed bound"));
            continue;
        }
        if flags.iter().any(|flag| !safe_flag(flag, profile.bounds.max_text_bytes)) {
            blockers.push(blocker(
                "unsafe-compiler-flag",
                subject,
                "compiler flags must be bounded single arguments without shell syntax",
            ));
        }
    }
}

fn validate_non_claims(profile: &ExperimentProfile, blockers: &mut Vec<ExperimentBlocker>) {
    if count_above_bound(profile.non_claims.len(), MAX_NON_CLAIMS) {
        blockers.push(blocker("non-claim-limit", "profile.non_claims", "profile non-claims exceed the fixed bound"));
        return;
    }
    for required in REQUIRED_NON_CLAIMS {
        if !profile.non_claims.iter().any(|value| value == required) {
            blockers.push(blocker(
                "missing-required-non-claim",
                *required,
                "KernelScript profile omits a required non-claim",
            ));
        }
    }
}

fn validate_artifact(artifact: &ArtifactIdentity, subject: &str, blockers: &mut Vec<ExperimentBlocker>) {
    let expected_ref = alloc::format!("{ARTIFACT_REF_PREFIX}{}", artifact.digest_blake3);
    if artifact.artifact_ref != expected_ref || artifact.size_bytes == 0 {
        blockers.push(blocker(
            "invalid-artifact-identity",
            subject,
            "artifact ref, BLAKE3, and positive size must agree",
        ));
    }
}

fn validate_text(validation: TextValidation<'_>, blockers: &mut Vec<ExperimentBlocker>) {
    let is_too_large =
        usize::try_from(validation.maximum_bytes).is_ok_and(|maximum_bytes| validation.value.len() > maximum_bytes);
    let has_control_character = validation.value.chars().any(char::is_control);
    if validation.value.is_empty() || is_too_large || has_control_character {
        blockers.push(blocker(
            "invalid-bounded-text",
            validation.subject,
            "text must be non-empty, bounded UTF-8 without control characters",
        ));
    }
}

fn validate_bound_u32(validation: BoundU32Validation<'_>, blockers: &mut Vec<ExperimentBlocker>) {
    if validation.value == 0 || validation.value > validation.maximum {
        blockers.push(blocker(
            "invalid-experiment-bound",
            validation.subject,
            "profile bound is zero or exceeds the hard experiment ceiling",
        ));
    }
}

fn validate_bound_u64(validation: BoundU64Validation<'_>, blockers: &mut Vec<ExperimentBlocker>) {
    if validation.value == 0 || validation.value > validation.maximum {
        blockers.push(blocker(
            "invalid-experiment-bound",
            validation.subject,
            "profile bound is zero or exceeds the hard experiment ceiling",
        ));
    }
}

fn identity_if_clean<T: Serialize>(value: &T, blockers: &mut Vec<ExperimentBlocker>) -> Option<Blake3Digest> {
    if !blockers.is_empty() {
        return None;
    }
    match canonical_identity(value) {
        Ok(identity) => Some(identity),
        Err(error) => {
            blockers.push(blocker("profile-identity-failed", "profile", &error.to_string()));
            None
        }
    }
}

fn safe_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && path.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}

fn safe_flag(flag: &str, maximum_bytes: u32) -> bool {
    let is_within_bound =
        usize::try_from(maximum_bytes).is_ok_and(|maximum_bytes| !flag.is_empty() && flag.len() <= maximum_bytes);
    is_within_bound && !flag.bytes().any(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b';' | b'|' | b'&' | b'`' | b'$'))
}

fn valid_kernel_build_identity(value: &str) -> bool {
    kernel_build_identity_digest(value).is_some()
}

pub(crate) fn kernel_build_identity_has_onix_authority(value: &str) -> bool {
    value
        .strip_prefix(ONIX_KERNEL_BUILD_IDENTITY_PREFIX)
        .is_some_and(|hex| lower_hex(hex, crate::BLAKE3_HEX_LENGTH))
}

fn kernel_build_identity_digest(value: &str) -> Option<&str> {
    value
        .strip_prefix(ONIX_KERNEL_BUILD_IDENTITY_PREFIX)
        .or_else(|| value.strip_prefix(OBSERVATION_KERNEL_BUILD_IDENTITY_PREFIX))
        .filter(|hex| lower_hex(hex, crate::BLAKE3_HEX_LENGTH))
}

fn lower_hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn tool_role_name(role: ToolRole) -> &'static str {
    match role {
        ToolRole::Ocaml => "ocaml",
        ToolRole::Dune => "dune",
        ToolRole::Menhir => "menhir",
        ToolRole::Clang => "clang",
        ToolRole::CCompiler => "c-compiler",
        ToolRole::Bpftool => "bpftool",
        ToolRole::Libbpf => "libbpf",
        ToolRole::ElfLibrary => "elf-library",
        ToolRole::Zlib => "zlib",
        ToolRole::KernelBuild => "kernel-build",
    }
}
