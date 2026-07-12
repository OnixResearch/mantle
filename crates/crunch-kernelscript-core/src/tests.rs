use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use super::*;

const TEST_ARTIFACT_SIZE_BYTES: u64 = 64;
const TEST_SOURCE_SIZE_BYTES: u64 = 128;
const TEST_MAX_GENERATED_FILES: u32 = 16;
const TEST_MAX_GENERATED_FILE_BYTES: u64 = 65_536;
const TEST_MAX_GENERATED_TOTAL_BYTES: u64 = 262_144;
const TEST_MAX_COMPILATION_STEPS: u32 = 16;
const TEST_MAX_OUTPUT_FILES: u32 = 8;
const TEST_MAX_OUTPUT_BYTES: u64 = 1_048_576;
const TEST_MAX_ELF_SECTIONS: u32 = 64;
const TEST_MAX_SECTION_NAME_BYTES: u32 = 128;
const TEST_MAX_RECEIPT_BLOCKERS: u32 = 32;
const TEST_MAX_TEXT_BYTES: u32 = 1_024;
const TEST_DEPENDENCY_COUNT: u32 = 4;
const ELF_HEADER_BYTES: usize = 64;
const ELF_SECTION_HEADER_BYTES: usize = 64;
const ELF_SECTION_TABLE_OFFSET: usize = ELF_HEADER_BYTES;
const ELF_SECTION_NAMES_INDEX: u16 = 1;
const ELF_FIRST_CONTENT_SECTION_INDEX: usize = 2;
const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
const ELF_CLASS_OFFSET: usize = 4;
const ELF_DATA_OFFSET: usize = 5;
const ELF_VERSION_OFFSET: usize = 6;
const ELF_TYPE_OFFSET: usize = 16;
const ELF_MACHINE_OFFSET: usize = 18;
const ELF_SECTION_TABLE_POINTER_OFFSET: usize = 40;
const ELF_SECTION_ENTRY_SIZE_OFFSET: usize = 58;
const ELF_SECTION_COUNT_OFFSET: usize = 60;
const ELF_SECTION_NAMES_INDEX_OFFSET: usize = 62;
const SECTION_NAME_OFFSET: usize = 0;
const SECTION_FILE_OFFSET: usize = 24;
const SECTION_SIZE_OFFSET: usize = 32;
const ELF_DATA_LITTLE_ENDIAN: u8 = 1;
const ELF_VERSION_CURRENT: u8 = 1;
const BTF_MAGIC: u16 = 0xeb9f;
const BTF_MAGIC_OFFSET: usize = 0;
const BTF_HEADER_BYTES: usize = 24;
const BTF_FIXTURE_BYTES: usize = BTF_HEADER_BYTES + 1;
const BTF_VERSION_OFFSET: usize = 2;
const BTF_FLAGS_OFFSET: usize = 3;
const BTF_HEADER_LENGTH_OFFSET: usize = 4;
const BTF_TYPE_OFFSET_OFFSET: usize = 8;
const BTF_TYPE_LENGTH_OFFSET: usize = 12;
const BTF_STRING_OFFSET_OFFSET: usize = 16;
const BTF_STRING_LENGTH_OFFSET: usize = 20;
const BTF_STRING_LENGTH: u32 = 1;
const BTF_FLAGS_NONE: u8 = 0;
const BTF_VERSION_CURRENT: u8 = 1;
const OFFICIAL_COMPILER_REVISION: &str = "0c80d4e4ac0029d34cbc9d65e76d78c075b64555";
const OFFICIAL_SOURCE_SHA256: &str = "9a00b96e1f127d4806c28b076f270acdc4bf4a8c558ca636bfd9f49268b479c1";
const OFFICIAL_SOURCE_BLAKE3: &str = "439431f81df45b043c218f4f5a41917ddd616e0defa35ff134c1cf5273124a57";
const OFFICIAL_SOURCE_URL: &str =
    "https://github.com/multikernel/kernelscript/releases/download/v0.1.2/kernelscript-0.1.2-source.tar.gz";
const KERNEL_BUILD_IDENTITY: &str =
    "onix:blake3:kernel-build:71ab5fe37dd010e284119879478a025fa85e07895041e3ddd91627b76609ccbc";
const KERNEL_RELEASE: &str = "6.12.0-onix-fixture";
const GENERATED_USERSPACE: &str = "demo.c";
const GENERATED_EBPF: &str = "demo.ebpf.c";
const GENERATED_MODULE: &str = "demo.mod.c";
const GENERATED_TEST: &str = "demo.test.c";
const GENERATED_MAKEFILE: &str = "Makefile";
const GENERATED_KBUILD: &str = "Kbuild";
const OUTPUT_EBPF: &str = "build/demo.ebpf.o";
const OUTPUT_USERSPACE: &str = "build/demo";
const OUTPUT_MODULE: &str = "build/demo.mod.ko";
const OUTPUT_TEST: &str = "build/demo.test";

fn digest(seed: char) -> Blake3Digest {
    Blake3Digest::parse(seed.to_string().repeat(BLAKE3_HEX_LENGTH)).unwrap()
}

fn artifact(seed: char) -> ArtifactIdentity {
    let digest_blake3 = digest(seed);
    ArtifactIdentity {
        artifact_ref: format!("mantle://blake3/{digest_blake3}"),
        digest_blake3,
        size_bytes: TEST_ARTIFACT_SIZE_BYTES,
    }
}

fn tool(role: ToolRole, seed: char) -> ToolIdentity {
    ToolIdentity {
        role,
        version: String::from("fixture-version"),
        executable_or_library: artifact(seed),
        configuration_blake3: digest(seed),
    }
}

fn compiler_dependency(package: &str, seed: char) -> CompilerDependency {
    CompilerDependency {
        package: String::from(package),
        version: String::from("fixture-only"),
        artifact: artifact(seed),
    }
}

fn compiler_dependencies() -> Vec<CompilerDependency> {
    vec![
        compiler_dependency("cmdliner", '1'),
        compiler_dependency("dune", '2'),
        compiler_dependency("menhir", '3'),
        compiler_dependency("menhirLib", '4'),
    ]
}

fn toolchain(include_kernel_build: bool) -> Vec<ToolIdentity> {
    let mut tools = vec![
        tool(ToolRole::Ocaml, '1'),
        tool(ToolRole::Dune, '2'),
        tool(ToolRole::Menhir, '3'),
        tool(ToolRole::Clang, '4'),
        tool(ToolRole::CCompiler, '5'),
        tool(ToolRole::Bpftool, '6'),
        tool(ToolRole::Libbpf, '7'),
        tool(ToolRole::ElfLibrary, '8'),
        tool(ToolRole::Zlib, '9'),
    ];
    if include_kernel_build {
        tools.push(tool(ToolRole::KernelBuild, 'a'));
    }
    tools
}

fn target_input(role: TargetInputRole, seed: char) -> TargetInputIdentity {
    TargetInputIdentity {
        role,
        artifact: artifact(seed),
        architecture: KernelArchitecture::X86_64,
        kernel_release: String::from(KERNEL_RELEASE),
        kernel_build_identity: String::from(KERNEL_BUILD_IDENTITY),
    }
}

fn expected_file(relative_path: &str, class: GeneratedFileClass) -> ExpectedGeneratedFile {
    ExpectedGeneratedFile {
        relative_path: String::from(relative_path),
        class,
        required: true,
        max_bytes: TEST_MAX_GENERATED_FILE_BYTES,
    }
}

fn bounds() -> ExperimentBounds {
    ExperimentBounds {
        max_generated_files: TEST_MAX_GENERATED_FILES,
        max_generated_file_bytes: TEST_MAX_GENERATED_FILE_BYTES,
        max_generated_total_bytes: TEST_MAX_GENERATED_TOTAL_BYTES,
        max_compilation_steps: TEST_MAX_COMPILATION_STEPS,
        max_output_files: TEST_MAX_OUTPUT_FILES,
        max_output_bytes: TEST_MAX_OUTPUT_BYTES,
        max_elf_sections: TEST_MAX_ELF_SECTIONS,
        max_section_name_bytes: TEST_MAX_SECTION_NAME_BYTES,
        max_receipt_blockers: TEST_MAX_RECEIPT_BLOCKERS,
        max_text_bytes: TEST_MAX_TEXT_BYTES,
    }
}

fn required_non_claims() -> Vec<String> {
    REQUIRED_NON_CLAIMS.iter().map(|value| String::from(*value)).collect()
}

fn profile() -> ExperimentProfile {
    ExperimentProfile {
        schema: String::from(EXPERIMENT_PROFILE_SCHEMA),
        experiment_id: String::from("fixture-only-userspace-probe"),
        beta: true,
        enabled_by_default: false,
        source: SourceIdentity {
            relative_path: String::from("demo.ks"),
            digest_blake3: Blake3Digest::from_slice(b"fixture-only KernelScript source"),
            size_bytes: TEST_SOURCE_SIZE_BYTES,
        },
        compiler: CompilerCohort {
            version: String::from("0.1.2"),
            source_revision: String::from(OFFICIAL_COMPILER_REVISION),
            source_archive_url: String::from(OFFICIAL_SOURCE_URL),
            source_archive_sha256: Sha256Digest::parse(String::from(OFFICIAL_SOURCE_SHA256)).unwrap(),
            source_archive_blake3: Blake3Digest::parse(String::from(OFFICIAL_SOURCE_BLAKE3)).unwrap(),
            dependency_lock: CompilerDependencyLock {
                format: String::from("opam-lock"),
                dependency_count: TEST_DEPENDENCY_COUNT,
                dependencies: compiler_dependencies(),
                lock_blake3: digest('b'),
            },
            compiler_executable: artifact('c'),
            closure_blake3: digest('d'),
        },
        toolchain: toolchain(false),
        target: KernelTarget {
            architecture: KernelArchitecture::X86_64,
            kernel_release: String::from(KERNEL_RELEASE),
            kernel_build_identity: String::from(KERNEL_BUILD_IDENTITY),
            btf: target_input(TargetInputRole::Btf, 'e'),
            headers: target_input(TargetInputRole::Headers, 'f'),
            config: target_input(TargetInputRole::Config, '0'),
        },
        output_classes: vec![
            ArtifactOutputClass::GeneratedSourceBundle,
            ArtifactOutputClass::UserspaceLoader,
            ArtifactOutputClass::EbpfObject,
            ArtifactOutputClass::CandidatePackManifest,
        ],
        expected_generated_files: vec![
            expected_file(GENERATED_USERSPACE, GeneratedFileClass::UserspaceC),
            expected_file(GENERATED_EBPF, GeneratedFileClass::EbpfC),
            expected_file(GENERATED_MAKEFILE, GeneratedFileClass::MakefileEvidence),
        ],
        bpf_compiler_flags: vec![
            String::from("-target"),
            String::from("bpf"),
            String::from("-O2"),
            String::from("-Wall"),
            String::from("-Wextra"),
            String::from("-g"),
            String::from("-fno-builtin"),
        ],
        userspace_compiler_flags: vec![
            String::from("-Wall"),
            String::from("-Wextra"),
            String::from("-O2"),
            String::from("-fPIC"),
            String::from("-lbpf"),
            String::from("-lelf"),
            String::from("-lz"),
        ],
        module_compiler_flags: vec![String::from("-Werror")],
        bounds: bounds(),
        network_policy: NetworkPolicy::Denied,
        non_claims: required_non_claims(),
    }
}

fn full_profile() -> ExperimentProfile {
    let mut profile = profile();
    profile.experiment_id = String::from("fixture-only-kfunc-module");
    profile.toolchain = toolchain(true);
    profile.output_classes.push(ArtifactOutputClass::KernelModule);
    profile.output_classes.push(ArtifactOutputClass::TestBinary);
    profile.expected_generated_files.push(expected_file(GENERATED_MODULE, GeneratedFileClass::ModuleC));
    profile
        .expected_generated_files
        .push(expected_file(GENERATED_KBUILD, GeneratedFileClass::KbuildEvidence));
    profile.expected_generated_files.push(expected_file(GENERATED_TEST, GeneratedFileClass::TestC));
    profile
}

fn compiler_facts(profile: &ExperimentProfile) -> CompilerMaterializationFacts {
    CompilerMaterializationFacts {
        source_revision: profile.compiler.source_revision.clone(),
        source_archive_sha256: profile.compiler.source_archive_sha256.clone(),
        source_archive_blake3: profile.compiler.source_archive_blake3.clone(),
        dependency_lock_blake3: Some(profile.compiler.dependency_lock.lock_blake3.clone()),
        dependency_count: profile.compiler.dependency_lock.dependency_count,
        dependencies: profile.compiler.dependency_lock.dependencies.clone(),
        compiler_executable: Some(profile.compiler.compiler_executable.clone()),
        closure_blake3: Some(profile.compiler.closure_blake3.clone()),
        network_attempted: false,
        ambient_opam_used: false,
        ambient_compiler_used: false,
    }
}

fn generated_files(include_module_and_test: bool) -> Vec<ObservedGeneratedFile> {
    let mut files = vec![
        generated(GENERATED_USERSPACE, b"int main(void) { return 0; }\n"),
        generated(GENERATED_EBPF, b"int probe(void *ctx) { return ctx != 0; }\n"),
        generated(GENERATED_MAKEFILE, b"all:\n\tclang demo.ebpf.c\n\tsudo ./demo\n"),
    ];
    if include_module_and_test {
        files.push(generated(GENERATED_MODULE, b"int fixture_kfunc(void) { return 0; }\n"));
        files.push(generated(GENERATED_KBUILD, b"obj-m += demo.mod.o\n"));
        files.push(generated(GENERATED_TEST, b"int main(void) { return 0; }\n"));
    }
    files
}

fn generated(path: &str, bytes: &[u8]) -> ObservedGeneratedFile {
    ObservedGeneratedFile {
        relative_path: String::from(path),
        bytes: bytes.to_vec(),
        executable: false,
    }
}

fn classified(profile: ExperimentProfile, include_module_and_test: bool) -> GeneratedProjectManifest {
    classify_generated_project(GeneratedProjectFacts {
        profile,
        files: generated_files(include_module_and_test),
    })
    .manifest
    .unwrap()
}

fn target_facts(profile: &ExperimentProfile) -> ResolvedKernelTargetFacts {
    ResolvedKernelTargetFacts {
        architecture: profile.target.architecture,
        kernel_release: profile.target.kernel_release.clone(),
        kernel_build_identity: profile.target.kernel_build_identity.clone(),
        btf: Some(profile.target.btf.artifact.clone()),
        headers: Some(profile.target.headers.artifact.clone()),
        config: Some(profile.target.config.artifact.clone()),
        compiler_flags_blake3: crate::target::expected_compiler_flags_identity(profile).unwrap(),
        toolchain_blake3: crate::target::expected_toolchain_identity(profile).unwrap(),
        ambient_inputs_used: false,
    }
}

#[test]
fn typed_beta_profile_has_stable_identity_and_official_source_pin() {
    let first = validate_profile(profile());
    let second = validate_profile(profile());

    assert!(first.blockers.is_empty(), "blockers: {:?}", first.blockers);
    assert_eq!(first.profile_identity_blake3, second.profile_identity_blake3);
    assert_eq!(profile().compiler.source_revision, OFFICIAL_COMPILER_REVISION);
    assert_eq!(profile().compiler.source_archive_sha256.as_str(), OFFICIAL_SOURCE_SHA256);
    assert!(!profile().enabled_by_default);
}

#[test]
fn profile_rejects_default_enable_missing_nonclaim_bad_bounds_and_unsupported_architecture_json() {
    let mut invalid = profile();
    invalid.enabled_by_default = true;
    invalid.non_claims.clear();
    invalid.bounds.max_generated_files = 0;
    invalid.compiler.dependency_lock.dependencies.clear();
    let validation = validate_profile(invalid);
    let unsupported = serde_json::from_str::<KernelArchitecture>("\"riscv64\"");

    assert!(validation.profile_identity_blake3.is_none());
    assert!(validation.blockers.iter().any(|item| item.code == "experiment-not-bounded-beta"));
    assert!(validation.blockers.iter().any(|item| item.code == "missing-required-non-claim"));
    assert!(validation.blockers.iter().any(|item| item.code == "invalid-experiment-bound"));
    assert!(validation.blockers.iter().any(|item| item.code == "incomplete-compiler-dependency-lock"));
    assert!(unsupported.is_err());
}

#[test]
fn compiler_materialization_admits_exact_offline_closure() {
    let profile = profile();
    let admission = admit_compiler_materialization(profile.clone(), compiler_facts(&profile));

    assert!(admission.admitted);
    assert!(admission.blockers.is_empty());
    assert!(admission.materialization_identity_blake3.is_some());
}

#[test]
fn compiler_materialization_rejects_source_dependency_network_and_ambient_drift() {
    let profile = profile();
    let mut facts = compiler_facts(&profile);
    facts.source_revision = String::from("0000000000000000000000000000000000000000");
    facts.dependency_lock_blake3 = None;
    facts.dependencies[0].version = String::from("drifted");
    facts.compiler_executable = None;
    facts.network_attempted = true;
    facts.ambient_opam_used = true;
    let admission = admit_compiler_materialization(profile, facts);

    assert!(!admission.admitted);
    assert!(admission.blockers.iter().any(|item| item.code == "compiler-source-drift"));
    assert!(admission.blockers.iter().any(|item| item.code == "compiler-dependency-closure-mismatch"));
    assert!(admission.blockers.iter().any(|item| item.code == "compiler-network-attempt"));
    assert!(admission.blockers.iter().any(|item| item.code == "ambient-compiler-state"));
}

#[test]
fn generated_project_classifier_accepts_exact_files_and_retains_makefile_only_as_evidence() {
    let profile = profile();
    let first = classify_generated_project(GeneratedProjectFacts {
        profile: profile.clone(),
        files: generated_files(false),
    });
    let second = classify_generated_project(GeneratedProjectFacts {
        profile,
        files: generated_files(false),
    });
    let manifest = first.manifest.unwrap();

    assert!(first.blockers.is_empty());
    assert_eq!(manifest.manifest_identity_blake3, second.manifest.unwrap().manifest_identity_blake3);
    assert!(manifest.members.iter().any(|member| member.class == GeneratedFileClass::MakefileEvidence));
    assert!(manifest.members.iter().all(|member| !member.executable));
}

#[test]
fn generated_project_classifier_rejects_missing_extra_escape_executable_and_oversized_files() {
    let profile = profile();
    let mut files = generated_files(false);
    files.retain(|file| file.relative_path != GENERATED_EBPF);
    files.push(generated("../escape.c", b"escape\n"));
    files.iter_mut().find(|file| file.relative_path == GENERATED_MAKEFILE).unwrap().executable = true;
    files.iter_mut().find(|file| file.relative_path == GENERATED_USERSPACE).unwrap().bytes =
        vec![b'x'; usize::try_from(TEST_MAX_GENERATED_FILE_BYTES).unwrap().saturating_add(1)];
    let result = classify_generated_project(GeneratedProjectFacts { profile, files });

    assert!(result.manifest.is_none());
    assert!(result.blockers.iter().any(|item| item.code == "missing-generated-file"));
    assert!(result.blockers.iter().any(|item| item.code == "unexpected-generated-file"));
    assert!(result.blockers.iter().any(|item| item.code == "generated-path-escape"));
    assert!(result.blockers.iter().any(|item| item.code == "generated-file-executable"));
    assert!(result.blockers.iter().any(|item| item.code == "generated-file-byte-bound"));
}

#[test]
fn codegen_and_compilation_plans_are_explicit_offline_and_never_consume_generated_build_scripts() {
    let profile = full_profile();
    let manifest = classified(profile.clone(), true);
    let codegen = plan_codegen(profile.clone()).unwrap();
    let compilation = plan_compilation(profile.clone(), manifest.clone()).plan.unwrap();
    let mut tampered_manifest = manifest;
    tampered_manifest.members[0].size_bytes = tampered_manifest.members[0].size_bytes.saturating_add(1);
    let rejected = plan_compilation(profile, tampered_manifest);

    assert_eq!(codegen.network_policy, NetworkPolicy::Denied);
    assert!(codegen.retain_generated_makefile_as_evidence);
    assert!(codegen.arguments.iter().any(|argument| argument == "--btf-vmlinux-path"));
    assert!(compilation.steps.iter().all(|step| !step.inputs.iter().any(|path| path == GENERATED_MAKEFILE)));
    assert!(compilation.steps.iter().all(|step| !step.inputs.iter().any(|path| path == GENERATED_KBUILD)));
    assert!(compilation.steps.iter().any(|step| step.action == PlanAction::WriteMantleModuleRecipe));
    assert!(rejected.plan.is_none());
    assert!(rejected.blockers.iter().any(|item| item.code == "generated-manifest-identity-mismatch"));
}

#[test]
fn execution_admission_allows_typed_tools_and_rejects_make_shell_unknown_paths_and_shell_syntax() {
    let positive = admit_execution_request(ExecutionRequest {
        program: String::from("clang"),
        arguments: vec![String::from("-target"), String::from("bpf")],
        input_paths: vec![String::from(GENERATED_EBPF)],
    });
    let generated_make = admit_execution_request(ExecutionRequest {
        program: String::from("make"),
        arguments: vec![String::from("all")],
        input_paths: vec![String::from(GENERATED_MAKEFILE)],
    });
    let shell = admit_execution_request(ExecutionRequest {
        program: String::from("/bin/sh"),
        arguments: vec![String::from("clang;curl")],
        input_paths: vec![String::from("../escape.c")],
    });

    assert!(positive.admitted);
    assert!(!generated_make.admitted);
    assert!(generated_make.blockers.iter().any(|item| item.code == "generated-build-script-authority"));
    assert!(generated_make.blockers.iter().any(|item| item.code == "unallowlisted-program"));
    assert!(!shell.admitted);
    assert!(shell.blockers.iter().any(|item| item.code == "unsafe-execution-argument"));
    assert!(shell.blockers.iter().any(|item| item.code == "unsafe-execution-input"));
}

#[test]
fn exact_kernel_target_admission_binds_btf_headers_config_flags_and_toolchain() {
    let profile = profile();
    let admission = admit_kernel_target(profile.clone(), target_facts(&profile));

    assert!(admission.admitted);
    assert!(admission.blockers.is_empty());
    assert!(admission.target_identity_blake3.is_some());
}

#[test]
fn kernel_target_admission_rejects_missing_ambient_mismatched_and_stale_facts() {
    let profile = profile();
    let mut facts = target_facts(&profile);
    facts.btf = None;
    facts.kernel_release = String::from("another-kernel");
    facts.compiler_flags_blake3 = digest('9');
    facts.toolchain_blake3 = digest('8');
    facts.ambient_inputs_used = true;
    let admission = admit_kernel_target(profile, facts);

    assert!(!admission.admitted);
    assert!(admission.blockers.iter().any(|item| item.code == "missing-or-mismatched-btf"));
    assert!(admission.blockers.iter().any(|item| item.code == "kernel-target-mismatch"));
    assert!(admission.blockers.iter().any(|item| item.code == "compiler-flag-cohort-mismatch"));
    assert!(admission.blockers.iter().any(|item| item.code == "target-toolchain-mismatch"));
    assert!(admission.blockers.iter().any(|item| item.code == "ambient-kernel-input"));
}

#[test]
fn each_selected_output_class_receives_independent_static_inspection() {
    let profile = full_profile();
    let manifest = classified(profile.clone(), true);
    let plan = plan_compilation(profile.clone(), manifest.clone()).plan.unwrap();
    let source_members = manifest.members.iter().map(|member| member.digest_blake3.clone()).collect::<Vec<_>>();
    let cases = vec![
        (ArtifactOutputClass::EbpfObject, OUTPUT_EBPF, elf(ELF_TYPE_RELOCATABLE, ELF_MACHINE_BPF, &[".BTF"])),
        (
            ArtifactOutputClass::UserspaceLoader,
            OUTPUT_USERSPACE,
            elf(ELF_TYPE_EXECUTABLE, ELF_MACHINE_X86_64, &[".text"]),
        ),
        (
            ArtifactOutputClass::KernelModule,
            OUTPUT_MODULE,
            elf(ELF_TYPE_RELOCATABLE, ELF_MACHINE_X86_64, &[".modinfo"]),
        ),
        (ArtifactOutputClass::TestBinary, OUTPUT_TEST, elf(ELF_TYPE_DYNAMIC, ELF_MACHINE_X86_64, &[".text"])),
    ];
    for (class, path, bytes) in cases {
        let result = inspect_output(
            profile.clone(),
            plan.clone(),
            observed_output(&profile, &plan, class, path, bytes, source_members.clone()),
        );
        assert!(result.blockers.is_empty(), "{class:?}: {:?}", result.blockers);
        assert!(result.inspection.unwrap().accepted);
    }
}

#[test]
fn static_inspection_rejects_missing_metadata_target_and_stale_plan() {
    let profile = full_profile();
    let manifest = classified(profile.clone(), true);
    let plan = plan_compilation(profile.clone(), manifest.clone()).plan.unwrap();
    let source_members = manifest.members.iter().map(|member| member.digest_blake3.clone()).collect::<Vec<_>>();
    let missing_btf = inspect_output(
        profile.clone(),
        plan.clone(),
        observed_output(
            &profile,
            &plan,
            ArtifactOutputClass::EbpfObject,
            OUTPUT_EBPF,
            elf(ELF_TYPE_RELOCATABLE, ELF_MACHINE_BPF, &[".text"]),
            source_members.clone(),
        ),
    );
    let wrong_module = inspect_output(
        profile.clone(),
        plan.clone(),
        observed_output(
            &profile,
            &plan,
            ArtifactOutputClass::KernelModule,
            OUTPUT_MODULE,
            elf(ELF_TYPE_RELOCATABLE, ELF_MACHINE_BPF, &[".BTF"]),
            source_members.clone(),
        ),
    );
    let mut stale = observed_output(
        &profile,
        &plan,
        ArtifactOutputClass::UserspaceLoader,
        OUTPUT_USERSPACE,
        vec![0_u8; ELF_HEADER_BYTES],
        source_members,
    );
    stale.plan_identity_blake3 = digest('f');
    stale.target_kernel_release = String::from("stale-release");
    let malformed = inspect_output(profile, plan, stale);

    assert!(missing_btf.blockers.iter().any(|item| item.code == "missing-bpf-btf-section"));
    assert!(wrong_module.blockers.iter().any(|item| item.code == "invalid-module-elf"));
    assert!(wrong_module.blockers.iter().any(|item| item.code == "missing-module-metadata"));
    assert!(malformed.blockers.iter().any(|item| item.code == "stale-output-plan"));
    assert!(malformed.blockers.iter().any(|item| item.code == "stale-output-target"));
    assert!(malformed.blockers.iter().any(|item| item.code == "malformed-elf-header"));
}

#[test]
fn static_inspection_rejects_malformed_btf_module_metadata_and_section_overflow() {
    let profile = full_profile();
    let manifest = classified(profile.clone(), true);
    let plan = plan_compilation(profile.clone(), manifest.clone()).plan.unwrap();
    let source_members = manifest.members.iter().map(|member| member.digest_blake3.clone()).collect::<Vec<_>>();
    let malformed_btf_bytes =
        elf_with_section_override(ELF_TYPE_RELOCATABLE, ELF_MACHINE_BPF, &[".BTF"], Some((".BTF", b"not-btf")));
    let malformed_btf = inspect_output(
        profile.clone(),
        plan.clone(),
        observed_output(
            &profile,
            &plan,
            ArtifactOutputClass::EbpfObject,
            OUTPUT_EBPF,
            malformed_btf_bytes,
            source_members.clone(),
        ),
    );
    let malformed_module_bytes = elf_with_section_override(
        ELF_TYPE_RELOCATABLE,
        ELF_MACHINE_X86_64,
        &[".modinfo"],
        Some((".modinfo", b"vermagic=another-kernel\0")),
    );
    let malformed_module = inspect_output(
        profile.clone(),
        plan.clone(),
        observed_output(
            &profile,
            &plan,
            ArtifactOutputClass::KernelModule,
            OUTPUT_MODULE,
            malformed_module_bytes,
            source_members.clone(),
        ),
    );
    let mut overflow_bytes = elf(ELF_TYPE_EXECUTABLE, ELF_MACHINE_X86_64, &[".text"]);
    write_u64(&mut overflow_bytes, ELF_SECTION_TABLE_POINTER_OFFSET, u64::MAX);
    let overflow = inspect_output(
        profile.clone(),
        plan.clone(),
        observed_output(
            &profile,
            &plan,
            ArtifactOutputClass::UserspaceLoader,
            OUTPUT_USERSPACE,
            overflow_bytes,
            source_members.clone(),
        ),
    );
    let mut wrong_source_observation = observed_output(
        &profile,
        &plan,
        ArtifactOutputClass::UserspaceLoader,
        OUTPUT_USERSPACE,
        elf(ELF_TYPE_EXECUTABLE, ELF_MACHINE_X86_64, &[".text"]),
        source_members,
    );
    wrong_source_observation.generated_source_members = vec![digest('f')];
    let wrong_source = inspect_output(profile, plan, wrong_source_observation);

    assert!(malformed_btf.blockers.iter().any(|item| item.code == "malformed-bpf-btf-metadata"));
    assert!(malformed_module.blockers.iter().any(|item| item.code == "malformed-module-metadata"));
    assert!(overflow.blockers.iter().any(|item| item.code == "malformed-elf-section-table"));
    assert!(wrong_source.blockers.iter().any(|item| item.code == "output-source-binding"));
}

#[test]
fn candidate_projections_remain_frontend_neutral_and_experimental_unverified() {
    let (profile, plan, inspections) = inspected_fixture();
    let result = project_candidate_packs(profile, plan, inspections);

    assert!(result.blockers.is_empty(), "blockers: {:?}", result.blockers);
    assert_eq!(result.projections.len(), 2);
    assert!(
        result
            .projections
            .iter()
            .all(|projection| projection.readiness == CandidateReadiness::ExperimentalUnverified)
    );
    assert!(
        result
            .projections
            .iter()
            .all(|projection| projection.non_claims.iter().any(|value| value == "onix-semantics-external"))
    );
    assert!(serde_json::from_str::<CandidateReadiness>("\"production-ready\"").is_err());
}

#[test]
fn candidate_projection_rejects_missing_or_unapproved_members_without_promoting_siblings() {
    let (profile, plan, mut inspections) = inspected_fixture();
    inspections.retain(|inspection| inspection.output_class != ArtifactOutputClass::KernelModule);
    inspections[0].accepted = false;
    let result = project_candidate_packs(profile, plan, inspections);

    assert!(result.blockers.iter().any(|item| item.code == "unadmitted-candidate-member"));
    assert!(result.blockers.iter().any(|item| item.code == "candidate-inspection-identity-mismatch"));
    assert!(result.blockers.iter().any(|item| item.code == "missing-candidate-members"));
    assert!(result.projections.is_empty());
}

#[test]
fn receipt_binds_exact_cohort_without_source_logs_host_paths_or_production_claims() {
    let input = inspected_receipt_input();
    let first = build_experiment_receipt(input.clone()).receipt.unwrap();
    let second = build_experiment_receipt(input).receipt.unwrap();
    let json = serde_json::to_string(&first).unwrap();

    assert_eq!(first.receipt_identity_blake3, second.receipt_identity_blake3);
    assert!(!json.contains("int main"));
    assert!(!json.contains("/home/"));
    assert!(!json.contains("/tmp/"));
    assert!(!json.contains("production-ready"));
    assert!(first.non_claims.iter().any(|value| value == "not-bpf-verifier-acceptance"));
}

#[test]
fn receipt_rejects_tampered_inspection_and_candidate_evidence_identities() {
    let mut inspection_tamper = inspected_receipt_input();
    inspection_tamper.output_inspections[0].section_names.push(String::from(".tampered"));
    let inspection_result = build_experiment_receipt(inspection_tamper);
    let mut candidate_tamper = inspected_receipt_input();
    candidate_tamper.candidate_packs[0].name.push_str("-tampered");
    let candidate_result = build_experiment_receipt(candidate_tamper);

    assert!(inspection_result.receipt.is_none());
    assert!(inspection_result.blockers.iter().any(|item| item.code == "receipt-inspection-mismatch"));
    assert!(candidate_result.receipt.is_none());
    assert!(candidate_result.blockers.iter().any(|item| item.code == "receipt-candidate-mismatch"));
}

#[test]
fn blocked_receipt_records_compile_failure_but_rejects_leaks_and_stage_overclaim() {
    let profile = profile();
    let compiler = admit_compiler_materialization(profile.clone(), compiler_facts(&profile));
    let blocker = ExperimentBlocker {
        code: String::from("ebpf-compile-failed"),
        subject: String::from("ebpf-object"),
        message: String::from("declared clang action exited nonzero"),
    };
    let output_class = OutputClassReceipt {
        output_class: ArtifactOutputClass::EbpfObject,
        status: OutputClassStatus::BuildFailed,
        members: Vec::new(),
        inspection_identities_blake3: Vec::new(),
        blocker_codes: vec![blocker.code.clone()],
    };
    let blocked = build_experiment_receipt(ExperimentReceiptInput {
        profile: profile.clone(),
        stage_status: ExperimentStageStatus::Blocked,
        compiler_admission: compiler.clone(),
        codegen_plan: Some(plan_codegen(profile.clone()).unwrap()),
        generated_manifest: None,
        target_admission: None,
        compilation_plan: None,
        output_classes: vec![output_class],
        output_inspections: Vec::new(),
        candidate_packs: Vec::new(),
        blockers: vec![blocker],
    });
    let leaked = build_experiment_receipt(ExperimentReceiptInput {
        profile: profile.clone(),
        stage_status: ExperimentStageStatus::Blocked,
        compiler_admission: compiler.clone(),
        codegen_plan: None,
        generated_manifest: None,
        target_admission: None,
        compilation_plan: None,
        output_classes: Vec::new(),
        output_inspections: Vec::new(),
        candidate_packs: Vec::new(),
        blockers: vec![ExperimentBlocker {
            code: String::from("failed"),
            subject: String::from("/home/operator/source"),
            message: String::from("token=secret"),
        }],
    });
    let overclaim = build_experiment_receipt(ExperimentReceiptInput {
        profile,
        stage_status: ExperimentStageStatus::Inspected,
        compiler_admission: compiler,
        codegen_plan: None,
        generated_manifest: None,
        target_admission: None,
        compilation_plan: None,
        output_classes: Vec::new(),
        output_inspections: Vec::new(),
        candidate_packs: Vec::new(),
        blockers: Vec::new(),
    });

    assert!(blocked.receipt.is_some());
    assert!(leaked.receipt.is_none());
    assert!(leaked.blockers.iter().any(|item| item.code == "receipt-sensitive-data"));
    assert!(overclaim.receipt.is_none());
    assert!(overclaim.blockers.iter().any(|item| item.code == "receipt-stage-status-overclaim"));
}

#[test]
fn receipt_and_candidate_dtos_reject_unknown_fields_and_production_vocabulary() {
    let unknown_receipt = serde_json::from_str::<ExperimentReceipt>(
        r#"{"schema":"mantle-kernelscript-experiment-receipt-v1","raw_log":"secret"}"#,
    );
    let production_status = serde_json::from_str::<ExperimentStageStatus>("\"production-ready\"");
    let production_output = serde_json::from_str::<OutputClassStatus>("\"deployable\"");

    assert!(unknown_receipt.is_err());
    assert!(production_status.is_err());
    assert!(production_output.is_err());
}

fn inspected_receipt_input() -> ExperimentReceiptInput {
    let (profile, plan, inspections) = inspected_fixture();
    let manifest = classified(profile.clone(), true);
    let compiler = admit_compiler_materialization(profile.clone(), compiler_facts(&profile));
    let codegen = plan_codegen(profile.clone()).unwrap();
    let target = admit_kernel_target(profile.clone(), target_facts(&profile));
    let candidates = project_candidate_packs(profile.clone(), plan.clone(), inspections.clone()).projections;
    let output_classes = output_class_receipts(&inspections);
    ExperimentReceiptInput {
        profile,
        stage_status: ExperimentStageStatus::Inspected,
        compiler_admission: compiler,
        codegen_plan: Some(codegen),
        generated_manifest: Some(manifest),
        target_admission: Some(target),
        compilation_plan: Some(plan),
        output_classes,
        output_inspections: inspections,
        candidate_packs: candidates,
        blockers: Vec::new(),
    }
}

fn inspected_fixture() -> (ExperimentProfile, CompilationPlan, Vec<OutputInspection>) {
    let profile = full_profile();
    let manifest = classified(profile.clone(), true);
    let plan = plan_compilation(profile.clone(), manifest.clone()).plan.unwrap();
    let source_members = manifest.members.iter().map(|member| member.digest_blake3.clone()).collect::<Vec<_>>();
    let observations = vec![
        observed_output(
            &profile,
            &plan,
            ArtifactOutputClass::EbpfObject,
            OUTPUT_EBPF,
            elf(ELF_TYPE_RELOCATABLE, ELF_MACHINE_BPF, &[".BTF"]),
            source_members.clone(),
        ),
        observed_output(
            &profile,
            &plan,
            ArtifactOutputClass::UserspaceLoader,
            OUTPUT_USERSPACE,
            elf(ELF_TYPE_EXECUTABLE, ELF_MACHINE_X86_64, &[".text"]),
            source_members.clone(),
        ),
        observed_output(
            &profile,
            &plan,
            ArtifactOutputClass::KernelModule,
            OUTPUT_MODULE,
            elf(ELF_TYPE_RELOCATABLE, ELF_MACHINE_X86_64, &[".modinfo"]),
            source_members.clone(),
        ),
        observed_output(
            &profile,
            &plan,
            ArtifactOutputClass::TestBinary,
            OUTPUT_TEST,
            elf(ELF_TYPE_DYNAMIC, ELF_MACHINE_X86_64, &[".text"]),
            source_members,
        ),
    ];
    let inspections = observations
        .into_iter()
        .map(|observed| inspect_output(profile.clone(), plan.clone(), observed).inspection.unwrap())
        .collect();
    (profile, plan, inspections)
}

fn observed_output(
    profile: &ExperimentProfile,
    plan: &CompilationPlan,
    output_class: ArtifactOutputClass,
    relative_path: &str,
    bytes: Vec<u8>,
    generated_project_members: Vec<Blake3Digest>,
) -> ObservedOutput {
    let generated_source_members =
        crate::inspection::expected_generated_source_members(plan, output_class, relative_path);
    debug_assert!(!generated_source_members.is_empty());
    debug_assert!(generated_source_members.iter().all(|member| generated_project_members.contains(member)));
    ObservedOutput {
        output_class,
        relative_path: String::from(relative_path),
        bytes,
        plan_identity_blake3: plan.plan_identity_blake3.clone(),
        target_kernel_build_identity: profile.target.kernel_build_identity.clone(),
        target_architecture: profile.target.architecture,
        target_kernel_release: profile.target.kernel_release.clone(),
        generated_source_members,
    }
}

fn output_class_receipts(inspections: &[OutputInspection]) -> Vec<OutputClassReceipt> {
    inspections
        .iter()
        .map(|inspection| OutputClassReceipt {
            output_class: inspection.output_class,
            status: OutputClassStatus::InspectionPassed,
            members: vec![OutputMember {
                relative_path: inspection.relative_path.clone(),
                digest_blake3: inspection.digest_blake3.clone(),
                size_bytes: inspection.size_bytes,
            }],
            inspection_identities_blake3: vec![inspection.inspection_identity_blake3.clone()],
            blocker_codes: Vec::new(),
        })
        .collect()
}

fn elf(elf_type: u16, machine: u16, required_sections: &[&str]) -> Vec<u8> {
    elf_with_section_override(elf_type, machine, required_sections, None)
}

fn elf_with_section_override(
    elf_type: u16,
    machine: u16,
    required_sections: &[&str],
    section_override: Option<(&str, &[u8])>,
) -> Vec<u8> {
    let mut section_names = vec![String::new(), String::from(".shstrtab")];
    section_names.extend(required_sections.iter().map(|name| String::from(*name)));
    let string_table = elf_string_table(&section_names);
    let name_offsets = elf_name_offsets(&section_names);
    let payloads = required_sections.iter().map(|name| section_payload(name, section_override)).collect::<Vec<_>>();
    let section_count = section_names.len();
    let table_bytes = ELF_SECTION_HEADER_BYTES.checked_mul(section_count).unwrap();
    let string_table_offset = ELF_SECTION_TABLE_OFFSET.checked_add(table_bytes).unwrap();
    let payload_offset = string_table_offset.checked_add(string_table.len()).unwrap();
    let payload_bytes = payloads.iter().try_fold(0_usize, |total, payload| total.checked_add(payload.len())).unwrap();
    let total_bytes = payload_offset.checked_add(payload_bytes).unwrap();
    let mut bytes = vec![0_u8; total_bytes];
    write_elf_header(&mut bytes, elf_type, machine, section_count);
    write_section_names(&mut bytes, &name_offsets, string_table_offset, &string_table);
    write_section_payloads(&mut bytes, payload_offset, &payloads);
    bytes
}

fn elf_string_table(section_names: &[String]) -> Vec<u8> {
    let mut string_table = vec![0_u8];
    for name in section_names.iter().skip(1) {
        string_table.extend_from_slice(name.as_bytes());
        string_table.push(0);
    }
    string_table
}

fn elf_name_offsets(section_names: &[String]) -> Vec<u32> {
    let mut offsets = Vec::with_capacity(section_names.len());
    let mut offset = 0_u32;
    for name in section_names {
        offsets.push(offset);
        let name_bytes = u32::try_from(name.len()).unwrap();
        offset = offset.checked_add(name_bytes).and_then(|value| value.checked_add(1)).unwrap();
    }
    offsets
}

fn write_elf_header(bytes: &mut [u8], elf_type: u16, machine: u16, section_count: usize) {
    bytes[..ELF_MAGIC.len()].copy_from_slice(&ELF_MAGIC);
    bytes[ELF_CLASS_OFFSET] = ELF_CLASS_64;
    bytes[ELF_DATA_OFFSET] = ELF_DATA_LITTLE_ENDIAN;
    bytes[ELF_VERSION_OFFSET] = ELF_VERSION_CURRENT;
    write_u16(bytes, ELF_TYPE_OFFSET, elf_type);
    write_u16(bytes, ELF_MACHINE_OFFSET, machine);
    write_u64(bytes, ELF_SECTION_TABLE_POINTER_OFFSET, u64::try_from(ELF_SECTION_TABLE_OFFSET).unwrap());
    write_u16(bytes, ELF_SECTION_ENTRY_SIZE_OFFSET, u16::try_from(ELF_SECTION_HEADER_BYTES).unwrap());
    write_u16(bytes, ELF_SECTION_COUNT_OFFSET, u16::try_from(section_count).unwrap());
    write_u16(bytes, ELF_SECTION_NAMES_INDEX_OFFSET, ELF_SECTION_NAMES_INDEX);
}

fn write_section_names(bytes: &mut [u8], name_offsets: &[u32], string_offset: usize, string_table: &[u8]) {
    for (index, name_offset) in name_offsets.iter().enumerate() {
        let header =
            ELF_SECTION_TABLE_OFFSET.checked_add(ELF_SECTION_HEADER_BYTES.checked_mul(index).unwrap()).unwrap();
        write_u32(bytes, header.checked_add(SECTION_NAME_OFFSET).unwrap(), *name_offset);
    }
    let names_header = ELF_SECTION_TABLE_OFFSET.checked_add(ELF_SECTION_HEADER_BYTES).unwrap();
    write_u64(bytes, names_header.checked_add(SECTION_FILE_OFFSET).unwrap(), u64::try_from(string_offset).unwrap());
    write_u64(
        bytes,
        names_header.checked_add(SECTION_SIZE_OFFSET).unwrap(),
        u64::try_from(string_table.len()).unwrap(),
    );
    let string_end = string_offset.checked_add(string_table.len()).unwrap();
    bytes[string_offset..string_end].copy_from_slice(string_table);
}

fn write_section_payloads(bytes: &mut [u8], mut payload_offset: usize, payloads: &[Vec<u8>]) {
    for (payload_index, payload) in payloads.iter().enumerate() {
        let section_index = ELF_FIRST_CONTENT_SECTION_INDEX.checked_add(payload_index).unwrap();
        let header = ELF_SECTION_TABLE_OFFSET
            .checked_add(ELF_SECTION_HEADER_BYTES.checked_mul(section_index).unwrap())
            .unwrap();
        write_u64(bytes, header.checked_add(SECTION_FILE_OFFSET).unwrap(), u64::try_from(payload_offset).unwrap());
        write_u64(bytes, header.checked_add(SECTION_SIZE_OFFSET).unwrap(), u64::try_from(payload.len()).unwrap());
        let payload_end = payload_offset.checked_add(payload.len()).unwrap();
        bytes[payload_offset..payload_end].copy_from_slice(payload);
        payload_offset = payload_end;
    }
}

fn section_payload(name: &str, section_override: Option<(&str, &[u8])>) -> Vec<u8> {
    if let Some((override_name, bytes)) = section_override
        && name == override_name
    {
        return bytes.to_vec();
    }
    match name {
        ".BTF" => btf_fixture(),
        ".modinfo" => format!("vermagic={KERNEL_RELEASE}\0").into_bytes(),
        _ => vec![0_u8],
    }
}

fn btf_fixture() -> Vec<u8> {
    let mut bytes = vec![0_u8; BTF_FIXTURE_BYTES];
    write_u16(&mut bytes, BTF_MAGIC_OFFSET, BTF_MAGIC);
    bytes[BTF_VERSION_OFFSET] = BTF_VERSION_CURRENT;
    bytes[BTF_FLAGS_OFFSET] = BTF_FLAGS_NONE;
    write_u32(&mut bytes, BTF_HEADER_LENGTH_OFFSET, u32::try_from(BTF_HEADER_BYTES).unwrap());
    write_u32(&mut bytes, BTF_TYPE_OFFSET_OFFSET, 0);
    write_u32(&mut bytes, BTF_TYPE_LENGTH_OFFSET, 0);
    write_u32(&mut bytes, BTF_STRING_OFFSET_OFFSET, 0);
    write_u32(&mut bytes, BTF_STRING_LENGTH_OFFSET, BTF_STRING_LENGTH);
    bytes
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    let encoded = value.to_le_bytes();
    bytes[offset..offset + encoded.len()].copy_from_slice(&encoded);
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    let encoded = value.to_le_bytes();
    bytes[offset..offset + encoded.len()].copy_from_slice(&encoded);
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    let encoded = value.to_le_bytes();
    bytes[offset..offset + encoded.len()].copy_from_slice(&encoded);
}
