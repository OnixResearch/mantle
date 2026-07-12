use alloc::collections::BTreeSet;
use alloc::string::String;
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
const KERNEL_BUILD_IDENTITY_PREFIX: &str = "onix:blake3:kernel-build:";
const SOURCE_EXTENSION: &str = ".ks";
const LOCK_FORMAT: &str = "opam-lock";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileValidation {
    pub profile_identity_blake3: Option<Blake3Digest>,
    pub blockers: Vec<ExperimentBlocker>,
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
    if profile.schema != EXPERIMENT_PROFILE_SCHEMA {
        blockers.push(blocker(
            "unsupported-profile-schema",
            "profile.schema",
            "KernelScript profile schema is unsupported",
        ));
    }
    validate_text(&profile.experiment_id, "profile.experiment_id", profile.bounds.max_text_bytes, blockers);
    if !profile.beta || profile.enabled_by_default {
        blockers.push(blocker(
            "experiment-not-bounded-beta",
            "profile",
            "KernelScript must remain beta and disabled by default",
        ));
    }
}

fn validate_source_and_compiler(profile: &ExperimentProfile, blockers: &mut Vec<ExperimentBlocker>) {
    if !safe_relative_path(&profile.source.relative_path) || !profile.source.relative_path.ends_with(SOURCE_EXTENSION) {
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
    validate_text(&compiler.version, "compiler.version", profile.bounds.max_text_bytes, blockers);
    if !lower_hex(&compiler.source_revision, GIT_REVISION_HEX_LENGTH) {
        blockers.push(blocker(
            "invalid-compiler-revision",
            "compiler.source_revision",
            "compiler revision must be exact Git hex",
        ));
    }
    if !compiler.source_archive_url.starts_with("https://") || compiler.source_archive_url.contains('@') {
        blockers.push(blocker(
            "invalid-compiler-source-url",
            "compiler.source_archive_url",
            "compiler source URL must be credential-free HTTPS",
        ));
    }
    validate_dependency_lock(&compiler.dependency_lock, profile.bounds.max_text_bytes, blockers);
    validate_artifact(&compiler.compiler_executable, "compiler.executable", blockers);
}

fn validate_dependency_lock(lock: &CompilerDependencyLock, text_bound: u32, blockers: &mut Vec<ExperimentBlocker>) {
    let count_matches = usize::try_from(lock.dependency_count).ok() == Some(lock.dependencies.len());
    if lock.format != LOCK_FORMAT
        || lock.dependencies.is_empty()
        || count_above_bound(lock.dependencies.len(), MAX_COMPILER_DEPENDENCIES)
        || !count_matches
    {
        blockers.push(blocker(
            "incomplete-compiler-dependency-lock",
            "compiler.dependency_lock",
            "compiler dependencies require an exact bounded non-empty opam-lock cohort",
        ));
        return;
    }
    let mut packages = BTreeSet::new();
    let mut artifacts = BTreeSet::new();
    for dependency in &lock.dependencies {
        validate_text(&dependency.package, "compiler.dependency.package", text_bound, blockers);
        validate_text(&dependency.version, "compiler.dependency.version", text_bound, blockers);
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
}

fn validate_bounds(bounds: &ExperimentBounds, blockers: &mut Vec<ExperimentBlocker>) {
    validate_bound_u32(bounds.max_generated_files, MAX_GENERATED_FILES_LIMIT, "max_generated_files", blockers);
    validate_bound_u64(
        bounds.max_generated_file_bytes,
        MAX_GENERATED_FILE_BYTES_LIMIT,
        "max_generated_file_bytes",
        blockers,
    );
    validate_bound_u64(
        bounds.max_generated_total_bytes,
        MAX_GENERATED_TOTAL_BYTES_LIMIT,
        "max_generated_total_bytes",
        blockers,
    );
    validate_bound_u32(bounds.max_compilation_steps, MAX_COMPILATION_STEPS_LIMIT, "max_compilation_steps", blockers);
    validate_bound_u32(bounds.max_output_files, MAX_OUTPUT_FILES_LIMIT, "max_output_files", blockers);
    validate_bound_u64(bounds.max_output_bytes, MAX_OUTPUT_BYTES_LIMIT, "max_output_bytes", blockers);
    validate_bound_u32(bounds.max_elf_sections, MAX_ELF_SECTIONS_LIMIT, "max_elf_sections", blockers);
    validate_bound_u32(bounds.max_section_name_bytes, MAX_SECTION_NAME_BYTES_LIMIT, "max_section_name_bytes", blockers);
    validate_bound_u32(bounds.max_receipt_blockers, MAX_RECEIPT_BLOCKERS_LIMIT, "max_receipt_blockers", blockers);
    validate_bound_u32(bounds.max_text_bytes, MAX_TEXT_BYTES_LIMIT, "max_text_bytes", blockers);
}

fn validate_toolchain(profile: &ExperimentProfile, blockers: &mut Vec<ExperimentBlocker>) {
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
            blockers.push(blocker("missing-tool-role", &tool_role_name(required), "required toolchain role is absent"));
        }
    }
}

fn validate_tool(tool: &ToolIdentity, text_bound: u32, blockers: &mut Vec<ExperimentBlocker>) {
    validate_text(&tool.version, "tool.version", text_bound, blockers);
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
    validate_artifact(&input.artifact, "target.input", blockers);
    if input.role != expected_role {
        blockers.push(blocker(
            "kernel-input-role-mismatch",
            "target.input",
            "kernel input occupies the wrong typed role",
        ));
    }
    if input.architecture != target.architecture
        || input.kernel_release != target.kernel_release
        || input.kernel_build_identity != target.kernel_build_identity
    {
        blockers.push(blocker(
            "kernel-input-cohort-mismatch",
            "target.input",
            "kernel input does not bind the exact target cohort",
        ));
    }
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
        if !safe_relative_path(&expected.relative_path)
            || expected.max_bytes == 0
            || expected.max_bytes > profile.bounds.max_generated_file_bytes
        {
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
                required,
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

fn validate_text(value: &str, subject: &str, maximum_bytes: u32, blockers: &mut Vec<ExperimentBlocker>) {
    let too_large = usize::try_from(maximum_bytes).is_ok_and(|maximum| value.len() > maximum);
    if value.is_empty() || too_large || value.chars().any(char::is_control) {
        blockers.push(blocker(
            "invalid-bounded-text",
            subject,
            "text must be non-empty, bounded UTF-8 without control characters",
        ));
    }
}

fn validate_bound_u32(value: u32, maximum: u32, subject: &str, blockers: &mut Vec<ExperimentBlocker>) {
    if value == 0 || value > maximum {
        blockers.push(blocker(
            "invalid-experiment-bound",
            subject,
            "profile bound is zero or exceeds the hard experiment ceiling",
        ));
    }
}

fn validate_bound_u64(value: u64, maximum: u64, subject: &str, blockers: &mut Vec<ExperimentBlocker>) {
    if value == 0 || value > maximum {
        blockers.push(blocker(
            "invalid-experiment-bound",
            subject,
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
    let within_bound = usize::try_from(maximum_bytes).is_ok_and(|maximum| !flag.is_empty() && flag.len() <= maximum);
    within_bound && !flag.bytes().any(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b';' | b'|' | b'&' | b'`' | b'$'))
}

fn valid_kernel_build_identity(value: &str) -> bool {
    value
        .strip_prefix(KERNEL_BUILD_IDENTITY_PREFIX)
        .is_some_and(|hex| lower_hex(hex, crate::BLAKE3_HEX_LENGTH))
}

fn lower_hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn tool_role_name(role: ToolRole) -> String {
    serde_json::to_string(&role)
        .unwrap_or_else(|_| String::from("unknown-tool-role"))
        .trim_matches('"')
        .to_string()
}
