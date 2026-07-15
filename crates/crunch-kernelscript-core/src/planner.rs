use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ArtifactOutputClass;
use crate::Blake3Digest;
use crate::ExperimentBlocker;
use crate::ExperimentProfile;
use crate::GeneratedFileClass;
use crate::GeneratedFileMember;
use crate::GeneratedProjectManifest;
use crate::KernelArchitecture;
use crate::NetworkPolicy;
use crate::ToolRole;
use crate::blocker::blocker;
use crate::digest::canonical_identity;
use crate::digest::count_above_bound;

pub const CODEGEN_PLAN_SCHEMA: &str = "mantle-kernelscript-codegen-plan-v1";
pub const COMPILATION_PLAN_SCHEMA: &str = "mantle-kernelscript-compilation-plan-v1";

const CODEGEN_OUTPUT_ROOT: &str = "generated";
const BUILD_OUTPUT_ROOT: &str = "build";
const MANTLE_MODULE_RECIPE: &str = "build/module/Kbuild";
const MAX_EXECUTION_ARGUMENTS: u32 = 64;
const MAX_EXECUTION_INPUTS: u32 = 16;
const MAX_PLAN_STEPS: usize = 7;
const PROGRAM_CLANG: &str = "clang";
const PROGRAM_CC: &str = "cc";
const PROGRAM_BPFTOOL: &str = "bpftool";
const PROGRAM_KERNEL_BUILD: &str = "kernel-build";
const PROGRAM_WRITE_MODULE_RECIPE: &str = "mantle-write-module-recipe";
const GENERATED_MAKEFILE: &str = "Makefile";
const GENERATED_KBUILD: &str = "Kbuild";
const MODULE_RECIPE_CONTENT_PREFIX: &str = "obj-m += ";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodegenPlan {
    pub schema: String,
    pub profile_identity_blake3: Blake3Digest,
    pub compiler_artifact_ref: String,
    pub source_relative_path: String,
    pub source_blake3: Blake3Digest,
    pub btf_artifact_ref: String,
    pub output_root: String,
    pub arguments: Vec<String>,
    pub network_policy: NetworkPolicy,
    pub retain_generated_makefile_as_evidence: bool,
    pub plan_identity_blake3: Blake3Digest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlanAction {
    GenerateVmlinuxHeader,
    CompileEbpf,
    GenerateSkeleton,
    CompileUserspace,
    WriteMantleModuleRecipe,
    CompileModule,
    CompileTest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanStep {
    pub step_id: String,
    pub output_class: ArtifactOutputClass,
    pub tool_role: Option<ToolRole>,
    pub action: PlanAction,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompilationPlan {
    pub schema: String,
    pub profile_identity_blake3: Blake3Digest,
    pub generated_manifest_identity_blake3: Blake3Digest,
    pub generated_members: Vec<GeneratedFileMember>,
    pub target_kernel_build_identity: String,
    pub target_architecture: KernelArchitecture,
    pub target_kernel_release: String,
    pub steps: Vec<PlanStep>,
    pub plan_identity_blake3: Blake3Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompilationPlanResult {
    pub plan: Option<CompilationPlan>,
    pub blockers: Vec<ExperimentBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionRequest {
    pub program: String,
    pub arguments: Vec<String>,
    pub input_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionAdmission {
    pub admitted: bool,
    pub blockers: Vec<ExperimentBlocker>,
}

#[derive(Serialize)]
struct CodegenIdentityInput {
    schema: String,
    profile_identity_blake3: Blake3Digest,
    compiler_artifact_ref: String,
    source_relative_path: String,
    source_blake3: Blake3Digest,
    btf_artifact_ref: String,
    output_root: String,
    arguments: Vec<String>,
    network_policy: NetworkPolicy,
    retain_generated_makefile_as_evidence: bool,
}

#[derive(Serialize)]
struct CompilationIdentityInput {
    schema: String,
    profile_identity_blake3: Blake3Digest,
    generated_manifest_identity_blake3: Blake3Digest,
    generated_members: Vec<GeneratedFileMember>,
    target_kernel_build_identity: String,
    target_architecture: KernelArchitecture,
    target_kernel_release: String,
    steps: Vec<PlanStep>,
}

struct PlanStepInput {
    step_id: &'static str,
    output_class: ArtifactOutputClass,
    tool_role: Option<ToolRole>,
    action: PlanAction,
    inputs: Vec<String>,
    outputs: Vec<String>,
    arguments: Vec<String>,
}

pub fn plan_codegen(profile: ExperimentProfile) -> Result<CodegenPlan, Vec<ExperimentBlocker>> {
    let validation = crate::validate_profile(profile.clone());
    if !validation.blockers.is_empty() {
        return Err(validation.blockers);
    }
    let Some(profile_identity) = validation.profile_identity_blake3 else {
        return Err(vec![blocker(
            "codegen-profile-identity-missing",
            "codegen",
            "validated profile did not produce a canonical identity",
        )]);
    };
    let arguments = codegen_arguments(&profile);
    let input = CodegenIdentityInput {
        schema: String::from(CODEGEN_PLAN_SCHEMA),
        profile_identity_blake3: profile_identity.clone(),
        compiler_artifact_ref: profile.compiler.compiler_executable.artifact_ref.clone(),
        source_relative_path: profile.source.relative_path.clone(),
        source_blake3: profile.source.digest_blake3.clone(),
        btf_artifact_ref: profile.target.btf.artifact.artifact_ref.clone(),
        output_root: String::from(CODEGEN_OUTPUT_ROOT),
        arguments: arguments.clone(),
        network_policy: profile.network_policy,
        retain_generated_makefile_as_evidence: true,
    };
    let identity = canonical_identity(&input)
        .map_err(|error| vec![blocker("codegen-plan-identity-failed", "codegen", &error.to_string())])?;
    debug_assert!(!arguments.is_empty());
    debug_assert!(arguments.iter().all(|argument| !argument.is_empty()));
    Ok(CodegenPlan {
        schema: input.schema,
        profile_identity_blake3: input.profile_identity_blake3,
        compiler_artifact_ref: input.compiler_artifact_ref,
        source_relative_path: input.source_relative_path,
        source_blake3: input.source_blake3,
        btf_artifact_ref: input.btf_artifact_ref,
        output_root: input.output_root,
        arguments,
        network_policy: input.network_policy,
        retain_generated_makefile_as_evidence: true,
        plan_identity_blake3: identity,
    })
}

pub fn plan_compilation(profile: ExperimentProfile, manifest: GeneratedProjectManifest) -> CompilationPlanResult {
    let validation = crate::validate_profile(profile.clone());
    let mut blockers = validation.blockers;
    let Some(profile_identity) = validation.profile_identity_blake3 else {
        return CompilationPlanResult { plan: None, blockers };
    };
    validate_manifest_binding(&profile_identity, &manifest, &mut blockers);
    let mut steps = build_steps(&profile, &manifest, &mut blockers);
    validate_steps(&profile, &steps, &mut blockers);
    if !blockers.is_empty() {
        return CompilationPlanResult { plan: None, blockers };
    }
    steps.sort_by(|left, right| left.step_id.cmp(&right.step_id));
    let plan = identify_compilation_plan(&profile, profile_identity, manifest, steps, &mut blockers);
    debug_assert!(blockers.is_empty() == plan.is_some());
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
    CompilationPlanResult { plan, blockers }
}

pub fn admit_execution_request(request: ExecutionRequest) -> ExecutionAdmission {
    let mut blockers = Vec::new();
    if !allowed_program(&request.program) {
        blockers.push(blocker(
            "unallowlisted-program",
            "execution.program",
            "program is outside the Mantle-owned KernelScript allowlist",
        ));
    }
    if count_above_bound(request.arguments.len(), MAX_EXECUTION_ARGUMENTS)
        || count_above_bound(request.input_paths.len(), MAX_EXECUTION_INPUTS)
    {
        blockers.push(blocker(
            "execution-request-limit",
            "execution",
            "execution request exceeds fixed argument or input bounds",
        ));
    }
    if request.arguments.iter().any(|argument| unsafe_argument(argument)) {
        blockers.push(blocker(
            "unsafe-execution-argument",
            "execution.arguments",
            "execution argument contains shell syntax or is empty",
        ));
    }
    if request.input_paths.iter().any(|path| !safe_plan_path(path)) {
        blockers.push(blocker(
            "unsafe-execution-input",
            "execution.inputs",
            "execution input path is absolute, escaping, or non-canonical",
        ));
    }
    if request.input_paths.iter().any(|path| path == GENERATED_MAKEFILE || path == GENERATED_KBUILD) {
        blockers.push(blocker(
            "generated-build-script-authority",
            "execution.inputs",
            "generated Makefile or Kbuild may not be executed",
        ));
    }
    debug_assert!(blockers.is_empty() == allowed_execution_shape(&request));
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
    ExecutionAdmission {
        admitted: blockers.is_empty(),
        blockers,
    }
}

fn codegen_arguments(profile: &ExperimentProfile) -> Vec<String> {
    vec![
        String::from("compile"),
        profile.source.relative_path.clone(),
        String::from("--output"),
        String::from(CODEGEN_OUTPUT_ROOT),
        String::from("--btf-vmlinux-path"),
        profile.target.btf.artifact.artifact_ref.clone(),
    ]
}

fn validate_manifest_binding(
    profile_identity: &Blake3Digest,
    manifest: &GeneratedProjectManifest,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    let initial_blocker_count: usize = blockers.len();
    if &manifest.profile_identity_blake3 != profile_identity {
        blockers.push(blocker(
            "generated-profile-mismatch",
            "generated-project",
            "generated manifest belongs to another profile",
        ));
    }
    if manifest.schema != crate::GENERATED_PROJECT_MANIFEST_SCHEMA {
        blockers.push(blocker(
            "generated-schema-mismatch",
            "generated-project",
            "generated manifest schema is unsupported",
        ));
    }
    if crate::generated::expected_generated_manifest_identity(manifest).as_ref()
        != Some(&manifest.manifest_identity_blake3)
    {
        blockers.push(blocker(
            "generated-manifest-identity-mismatch",
            "generated-project",
            "generated manifest fields differ from their BLAKE3 identity",
        ));
    }
    debug_assert!(!profile_identity.as_str().is_empty());
    debug_assert!(blockers.len() >= initial_blocker_count);
}

fn build_steps(
    profile: &ExperimentProfile,
    manifest: &GeneratedProjectManifest,
    blockers: &mut Vec<ExperimentBlocker>,
) -> Vec<PlanStep> {
    let mut steps = Vec::with_capacity(MAX_PLAN_STEPS);
    let base = source_stem(&profile.source.relative_path);
    let needs_bpf = profile.output_classes.iter().any(|class| {
        matches!(
            class,
            ArtifactOutputClass::EbpfObject | ArtifactOutputClass::UserspaceLoader | ArtifactOutputClass::TestBinary
        )
    });
    if needs_bpf {
        append_bpf_steps(profile, manifest, &base, &mut steps, blockers);
    }
    if profile.output_classes.contains(&ArtifactOutputClass::UserspaceLoader) {
        append_userspace_step(profile, manifest, &base, &mut steps, blockers);
    }
    if profile.output_classes.contains(&ArtifactOutputClass::KernelModule) {
        append_module_steps(profile, manifest, &base, &mut steps, blockers);
    }
    if profile.output_classes.contains(&ArtifactOutputClass::TestBinary) {
        append_test_step(profile, manifest, &base, &mut steps, blockers);
    }
    debug_assert!(steps.len() <= MAX_PLAN_STEPS);
    debug_assert!(steps.iter().all(|planned| !planned.outputs.is_empty()));
    steps
}

fn append_bpf_steps(
    profile: &ExperimentProfile,
    manifest: &GeneratedProjectManifest,
    base: &str,
    steps: &mut Vec<PlanStep>,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    debug_assert!(steps.len() <= MAX_PLAN_STEPS);
    debug_assert!(!base.is_empty());
    let Some(source) = unique_member_path(manifest, GeneratedFileClass::EbpfC, blockers) else {
        return;
    };
    let vmlinux_header = format!("{BUILD_OUTPUT_ROOT}/vmlinux.h");
    let object = format!("{BUILD_OUTPUT_ROOT}/{base}.ebpf.o");
    let skeleton = format!("{BUILD_OUTPUT_ROOT}/{base}.skel.h");
    steps.push(step(PlanStepInput {
        step_id: "01-vmlinux-header",
        output_class: ArtifactOutputClass::EbpfObject,
        tool_role: Some(ToolRole::Bpftool),
        action: PlanAction::GenerateVmlinuxHeader,
        inputs: vec![profile.target.btf.artifact.artifact_ref.clone()],
        outputs: vec![vmlinux_header.clone()],
        arguments: vec![
            String::from("btf"),
            String::from("dump"),
            String::from("format"),
            String::from("c"),
        ],
    }));
    let mut bpf_arguments = profile.bpf_compiler_flags.clone();
    bpf_arguments.push(target_arch_define(profile.target.architecture));
    steps.push(step(PlanStepInput {
        step_id: "02-ebpf-object",
        output_class: ArtifactOutputClass::EbpfObject,
        tool_role: Some(ToolRole::Clang),
        action: PlanAction::CompileEbpf,
        inputs: vec![source, vmlinux_header],
        outputs: vec![object.clone()],
        arguments: bpf_arguments,
    }));
    steps.push(step(PlanStepInput {
        step_id: "03-bpf-skeleton",
        output_class: ArtifactOutputClass::EbpfObject,
        tool_role: Some(ToolRole::Bpftool),
        action: PlanAction::GenerateSkeleton,
        inputs: vec![object],
        outputs: vec![skeleton],
        arguments: vec![String::from("gen"), String::from("skeleton")],
    }));
}

fn append_userspace_step(
    profile: &ExperimentProfile,
    manifest: &GeneratedProjectManifest,
    base: &str,
    steps: &mut Vec<PlanStep>,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    let Some(source) = unique_member_path(manifest, GeneratedFileClass::UserspaceC, blockers) else {
        return;
    };
    steps.push(step(PlanStepInput {
        step_id: "04-userspace-loader",
        output_class: ArtifactOutputClass::UserspaceLoader,
        tool_role: Some(ToolRole::CCompiler),
        action: PlanAction::CompileUserspace,
        inputs: vec![source, format!("{BUILD_OUTPUT_ROOT}/{base}.skel.h")],
        outputs: vec![format!("{BUILD_OUTPUT_ROOT}/{base}")],
        arguments: profile.userspace_compiler_flags.clone(),
    }));
}

fn append_module_steps(
    profile: &ExperimentProfile,
    manifest: &GeneratedProjectManifest,
    base: &str,
    steps: &mut Vec<PlanStep>,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    debug_assert!(steps.len() <= MAX_PLAN_STEPS);
    debug_assert!(!base.is_empty());
    let Some(source) = unique_member_path(manifest, GeneratedFileClass::ModuleC, blockers) else {
        return;
    };
    let recipe_content = format!("{MODULE_RECIPE_CONTENT_PREFIX}{base}.mod.o\n");
    let recipe_digest = Blake3Digest::from_slice(recipe_content.as_bytes());
    steps.push(step(PlanStepInput {
        step_id: "05-module-recipe",
        output_class: ArtifactOutputClass::KernelModule,
        tool_role: None,
        action: PlanAction::WriteMantleModuleRecipe,
        inputs: Vec::new(),
        outputs: vec![String::from(MANTLE_MODULE_RECIPE)],
        arguments: vec![recipe_digest.into_hex()],
    }));
    let mut arguments = profile.module_compiler_flags.clone();
    arguments.push(String::from("modules"));
    steps.push(step(PlanStepInput {
        step_id: "06-kernel-module",
        output_class: ArtifactOutputClass::KernelModule,
        tool_role: Some(ToolRole::KernelBuild),
        action: PlanAction::CompileModule,
        inputs: vec![
            source,
            String::from(MANTLE_MODULE_RECIPE),
            profile.target.headers.artifact.artifact_ref.clone(),
        ],
        outputs: vec![format!("{BUILD_OUTPUT_ROOT}/{base}.mod.ko")],
        arguments,
    }));
}

fn append_test_step(
    profile: &ExperimentProfile,
    manifest: &GeneratedProjectManifest,
    base: &str,
    steps: &mut Vec<PlanStep>,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    let Some(source) = unique_member_path(manifest, GeneratedFileClass::TestC, blockers) else {
        return;
    };
    steps.push(step(PlanStepInput {
        step_id: "07-test-binary",
        output_class: ArtifactOutputClass::TestBinary,
        tool_role: Some(ToolRole::CCompiler),
        action: PlanAction::CompileTest,
        inputs: vec![source, format!("{BUILD_OUTPUT_ROOT}/{base}.ebpf.o")],
        outputs: vec![format!("{BUILD_OUTPUT_ROOT}/{base}.test")],
        arguments: profile.userspace_compiler_flags.clone(),
    }));
}

fn unique_member_path(
    manifest: &GeneratedProjectManifest,
    class: GeneratedFileClass,
    blockers: &mut Vec<ExperimentBlocker>,
) -> Option<String> {
    let matches = manifest.members.iter().filter(|member| member.class == class).collect::<Vec<_>>();
    if matches.len() != 1 {
        blockers.push(blocker(
            "generated-class-cardinality",
            "generated-project",
            "selected generated source class must have exactly one member",
        ));
        return None;
    }
    Some(matches[0].relative_path.clone())
}

fn validate_steps(profile: &ExperimentProfile, steps: &[PlanStep], blockers: &mut Vec<ExperimentBlocker>) {
    let initial_blocker_count: usize = blockers.len();
    if steps.is_empty() || count_above_bound(steps.len(), profile.bounds.max_compilation_steps) {
        blockers.push(blocker(
            "compilation-step-limit",
            "compilation-plan",
            "compilation plan must contain bounded explicit steps",
        ));
        return;
    }
    let mut ids = BTreeSet::new();
    for planned in steps {
        if !ids.insert(planned.step_id.clone()) {
            blockers.push(blocker("duplicate-compilation-step", &planned.step_id, "compilation step id is duplicated"));
        }
        if planned.inputs.iter().any(|path| path == GENERATED_MAKEFILE || path == GENERATED_KBUILD) {
            blockers.push(blocker(
                "generated-build-script-authority",
                &planned.step_id,
                "plan attempts to consume a generated build script",
            ));
        }
        if planned.inputs.iter().chain(planned.outputs.iter()).any(|path| !safe_plan_input(path)) {
            blockers.push(blocker(
                "unsafe-compilation-path",
                &planned.step_id,
                "compilation plan contains an unsafe path or artifact ref",
            ));
        }
    }
    debug_assert!(ids.len() <= steps.len());
    debug_assert!(blockers.len() >= initial_blocker_count);
}

fn identify_compilation_plan(
    profile: &ExperimentProfile,
    profile_identity: Blake3Digest,
    manifest: GeneratedProjectManifest,
    steps: Vec<PlanStep>,
    blockers: &mut Vec<ExperimentBlocker>,
) -> Option<CompilationPlan> {
    debug_assert!(!steps.is_empty());
    debug_assert!(steps.len() <= MAX_PLAN_STEPS);
    let input = CompilationIdentityInput {
        schema: String::from(COMPILATION_PLAN_SCHEMA),
        profile_identity_blake3: profile_identity.clone(),
        generated_manifest_identity_blake3: manifest.manifest_identity_blake3.clone(),
        generated_members: manifest.members.clone(),
        target_kernel_build_identity: profile.target.kernel_build_identity.clone(),
        target_architecture: profile.target.architecture,
        target_kernel_release: profile.target.kernel_release.clone(),
        steps: steps.clone(),
    };
    let identity = match canonical_identity(&input) {
        Ok(identity) => identity,
        Err(error) => {
            blockers.push(blocker("compilation-plan-identity-failed", "compilation-plan", &error.to_string()));
            return None;
        }
    };
    Some(CompilationPlan {
        schema: input.schema,
        profile_identity_blake3: input.profile_identity_blake3,
        generated_manifest_identity_blake3: input.generated_manifest_identity_blake3,
        generated_members: input.generated_members,
        target_kernel_build_identity: input.target_kernel_build_identity,
        target_architecture: input.target_architecture,
        target_kernel_release: input.target_kernel_release,
        steps,
        plan_identity_blake3: identity,
    })
}

fn step(input: PlanStepInput) -> PlanStep {
    debug_assert!(!input.step_id.is_empty());
    debug_assert!(!input.outputs.is_empty());
    PlanStep {
        step_id: String::from(input.step_id),
        output_class: input.output_class,
        tool_role: input.tool_role,
        action: input.action,
        inputs: input.inputs,
        outputs: input.outputs,
        arguments: input.arguments,
    }
}

fn source_stem(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).strip_suffix(".ks").unwrap_or(path).to_string()
}

fn target_arch_define(architecture: KernelArchitecture) -> String {
    match architecture {
        KernelArchitecture::X86_64 => String::from("-D__TARGET_ARCH_x86"),
        KernelArchitecture::Aarch64 => String::from("-D__TARGET_ARCH_arm64"),
    }
}

fn allowed_program(program: &str) -> bool {
    matches!(
        program,
        PROGRAM_CLANG | PROGRAM_CC | PROGRAM_BPFTOOL | PROGRAM_KERNEL_BUILD | PROGRAM_WRITE_MODULE_RECIPE
    )
}

fn allowed_execution_shape(request: &ExecutionRequest) -> bool {
    allowed_program(&request.program)
        && !count_above_bound(request.arguments.len(), MAX_EXECUTION_ARGUMENTS)
        && !count_above_bound(request.input_paths.len(), MAX_EXECUTION_INPUTS)
        && request.arguments.iter().all(|argument| !unsafe_argument(argument))
        && request.input_paths.iter().all(|path| safe_plan_path(path))
        && request.input_paths.iter().all(|path| path != GENERATED_MAKEFILE && path != GENERATED_KBUILD)
}

fn unsafe_argument(argument: &str) -> bool {
    argument.is_empty() || argument.bytes().any(|byte| matches!(byte, b'\n' | b'\r' | b';' | b'|' | b'&' | b'`' | b'$'))
}

pub(crate) fn expected_codegen_plan_identity(plan: &CodegenPlan) -> Option<Blake3Digest> {
    canonical_identity(&CodegenIdentityInput {
        schema: plan.schema.clone(),
        profile_identity_blake3: plan.profile_identity_blake3.clone(),
        compiler_artifact_ref: plan.compiler_artifact_ref.clone(),
        source_relative_path: plan.source_relative_path.clone(),
        source_blake3: plan.source_blake3.clone(),
        btf_artifact_ref: plan.btf_artifact_ref.clone(),
        output_root: plan.output_root.clone(),
        arguments: plan.arguments.clone(),
        network_policy: plan.network_policy,
        retain_generated_makefile_as_evidence: plan.retain_generated_makefile_as_evidence,
    })
    .ok()
}

pub(crate) fn expected_compilation_plan_identity(plan: &CompilationPlan) -> Option<Blake3Digest> {
    canonical_identity(&CompilationIdentityInput {
        schema: plan.schema.clone(),
        profile_identity_blake3: plan.profile_identity_blake3.clone(),
        generated_manifest_identity_blake3: plan.generated_manifest_identity_blake3.clone(),
        generated_members: plan.generated_members.clone(),
        target_kernel_build_identity: plan.target_kernel_build_identity.clone(),
        target_architecture: plan.target_architecture,
        target_kernel_release: plan.target_kernel_release.clone(),
        steps: plan.steps.clone(),
    })
    .ok()
}

fn safe_plan_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && path.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}

fn safe_plan_input(value: &str) -> bool {
    if value.starts_with("mantle://blake3/") {
        return value.len() == "mantle://blake3/".len().saturating_add(crate::BLAKE3_HEX_LENGTH);
    }
    safe_plan_path(value)
}
