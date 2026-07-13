use std::fs;
use std::path::Path;

use crunch_kernelscript_core::ArtifactOutputClass;
use crunch_kernelscript_core::ExperimentBlocker;
use crunch_kernelscript_core::ExperimentBounds;
use crunch_kernelscript_core::GeneratedFileClass;
use crunch_kernelscript_core::GeneratedProjectFacts;
use crunch_kernelscript_core::KernelArchitecture;
use crunch_kernelscript_core::ObservedGeneratedFile;
use crunch_kernelscript_core::ToolRole;
use crunch_kernelscript_core::classify_generated_project;
use sha2::Digest as Sha2Digest;
use sha2::Sha256;
use tempfile::TempDir;

use super::*;

const TEST_MAX_GENERATED_FILES: u32 = 8;
const TEST_MAX_GENERATED_FILE_BYTES: u64 = 65_536;
const TEST_MAX_GENERATED_TOTAL_BYTES: u64 = 262_144;
const TEST_MAX_COMPILATION_STEPS: u32 = 16;
const TEST_MAX_OUTPUT_FILES: u32 = 8;
const TEST_MAX_OUTPUT_BYTES: u64 = 1_048_576;
const TEST_MAX_ELF_SECTIONS: u32 = 64;
const TEST_MAX_SECTION_NAME_BYTES: u32 = 128;
const TEST_MAX_RECEIPT_BLOCKERS: u32 = 32;
const TEST_MAX_TEXT_BYTES: u32 = 1_024;
const SOURCE_REVISION: &str = "0c80d4e4ac0029d34cbc9d65e76d78c075b64555";
const INVALID_SOURCE_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const SOURCE_URL: &str =
    "https://github.com/multikernel/kernelscript/releases/download/v0.1.2/kernelscript-0.1.2-source.tar.gz";
const KERNEL_RELEASE: &str = "6.18.20";
const MODULE_BLOCKER_CODE: &str = "module-build-and-vm-gate-absent";

struct Fixture {
    _root: TempDir,
    generated_root: std::path::PathBuf,
    request: CoreAdapterRequest,
}

#[test]
fn exact_generated_shape_is_admitted_by_core_and_receipt_stays_observation_blocked() {
    let fixture = fixture(false);
    let first = run_request(fixture.request.clone(), &fixture.generated_root).unwrap();
    let second = run_request(fixture.request, &fixture.generated_root).unwrap();

    assert_eq!(first.generated_manifest.members.len(), 3);
    assert!(first.compiler_admission.admitted);
    assert!(!first.target_admission.admitted);
    assert_eq!(first.receipt.receipt_identity_blake3, second.receipt.receipt_identity_blake3);
    assert!(first.receipt.blockers.iter().any(|blocker| blocker.code == TARGET_OBSERVATION_BLOCKER_CODE));
    assert!(first.receipt.non_claims.iter().any(|claim| claim == "not-production-readiness"));
    assert!(first.receipt.candidate_packs.is_empty());
}

#[test]
fn adapter_manifest_matches_direct_core_classification() {
    let fixture = fixture(false);
    let report = run_request(fixture.request, &fixture.generated_root).unwrap();
    let direct = classify_generated_project(GeneratedProjectFacts {
        profile: report.profile.clone(),
        files: observed_fixture_files(&fixture.generated_root),
    });

    assert!(direct.blockers.is_empty(), "blockers: {:?}", direct.blockers);
    assert_eq!(Some(report.generated_manifest), direct.manifest);
    assert_eq!(report.compilation_plan.profile_identity_blake3, report.receipt.profile_identity_blake3);
    assert_eq!(
        report.compilation_plan.plan_identity_blake3,
        report.receipt.compilation_plan_identity_blake3.unwrap()
    );
}

#[test]
fn exact_shape_mismatch_is_rejected_by_core() {
    let fixture = fixture(false);
    fs::write(fixture.generated_root.join("unexpected.c"), b"int unexpected(void) { return 1; }\n").unwrap();
    let error = run_request(fixture.request, &fixture.generated_root).unwrap_err();
    let AdapterError::CoreRejected(blockers) = error else {
        panic!("unexpected adapter error: {error:?}");
    };

    assert!(blockers.iter().any(|blocker| blocker.code == "unexpected-generated-file"));
    assert!(!blockers.iter().any(|blocker| blocker.code == TARGET_OBSERVATION_BLOCKER_CODE));
}

#[test]
fn request_bounds_are_rejected_before_any_source_read() {
    let mut fixture = fixture(false);
    fixture.request.bounds.max_generated_file_bytes = u64::MAX;
    fixture.request.source.path = String::from("/missing/source-that-must-not-be-read.ks");
    let error = run_request(fixture.request, &fixture.generated_root).unwrap_err();

    assert!(matches!(error, AdapterError::Invalid(_)));
    assert!(error.to_string().contains("hard generated/output/text bounds"));
}

#[test]
fn generated_count_and_total_bytes_are_capped_before_allocation() {
    let mut count_fixture = fixture(false);
    count_fixture.request.bounds.max_generated_files = 3;
    fs::write(count_fixture.generated_root.join("extra.c"), b"extra\n").unwrap();
    let count_error = run_request(count_fixture.request, &count_fixture.generated_root).unwrap_err();
    let mut total_fixture = fixture(false);
    total_fixture.request.bounds.max_generated_total_bytes = 1;
    let total_error = run_request(total_fixture.request, &total_fixture.generated_root).unwrap_err();

    assert!(matches!(count_error, AdapterError::Invalid(_)));
    assert!(count_error.to_string().contains("file count"));
    assert!(matches!(total_error, AdapterError::Invalid(_)));
    assert!(total_error.to_string().contains("total bytes"));
}

#[test]
fn actual_source_archive_sha256_must_match_the_declared_interoperability_digest() {
    let mut fixture = fixture(false);
    fixture.request.compiler.source_archive_sha256 = String::from(INVALID_SOURCE_SHA256);
    let error = run_request(fixture.request, &fixture.generated_root).unwrap_err();
    let AdapterError::CoreRejected(blockers) = error else {
        panic!("unexpected adapter error: {error:?}");
    };

    assert!(blockers.iter().any(|blocker| blocker.code == "compiler-source-drift"));
    assert!(!blockers.iter().any(|blocker| blocker.code == TARGET_OBSERVATION_BLOCKER_CODE));
}

#[cfg(unix)]
#[test]
fn no_follow_reads_reject_source_and_generated_symlinks() {
    use std::os::unix::fs::symlink;

    let mut source_fixture = fixture(false);
    let source_link = source_fixture._root.path().join("source-link.ks");
    symlink(&source_fixture.request.source.path, &source_link).unwrap();
    source_fixture.request.source.path = source_link.to_str().unwrap().to_string();
    let source_error = run_request(source_fixture.request, &source_fixture.generated_root).unwrap_err();
    let generated_fixture = fixture(false);
    symlink(generated_fixture.generated_root.join("demo.c"), generated_fixture.generated_root.join("linked.c"))
        .unwrap();
    let generated_error = run_request(generated_fixture.request, &generated_fixture.generated_root).unwrap_err();

    assert!(matches!(source_error, AdapterError::Io { .. }));
    assert!(source_error.to_string().contains("no-follow"));
    assert!(matches!(generated_error, AdapterError::Invalid(_)));
    assert!(generated_error.to_string().contains("bounded regular data"));
}

#[test]
fn opened_descriptor_cannot_be_replaced_by_a_later_path_swap() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("identity");
    let old_path = root.path().join("identity-opened");
    let original = b"original descriptor bytes";
    fs::write(&path, original).unwrap();
    let opened = open_regular_file_no_follow(&path).unwrap();
    fs::rename(&path, &old_path).unwrap();
    fs::write(&path, b"replacement path bytes").unwrap();
    let observed = read_open_regular_file_bounded(opened, &path, TEST_MAX_GENERATED_FILE_BYTES).unwrap();

    assert_eq!(observed, original);
    assert_ne!(observed, fs::read(&path).unwrap());
}

#[test]
fn external_module_blocker_is_preserved_without_authority_promotion() {
    let fixture = fixture(true);
    let report = run_request(fixture.request, &fixture.generated_root).unwrap();

    assert!(report.receipt.blockers.iter().any(|blocker| blocker.code == MODULE_BLOCKER_CODE));
    assert!(report.receipt.blockers.iter().any(|blocker| blocker.code == TARGET_OBSERVATION_BLOCKER_CODE));
    assert!(!report.target_admission.admitted);
    assert!(report.receipt.candidate_packs.is_empty());
}

fn fixture(include_module: bool) -> Fixture {
    let root = tempfile::tempdir().unwrap();
    let source = write_file(root.path(), "demo.ks", b"fn main() -> i32 { return 0 }\n");
    let archive = write_file(root.path(), "source.tar.gz", b"pinned source archive bytes");
    let compiler = write_file(root.path(), "kernelscript", b"pinned compiler executable bytes");
    let closure = write_file(root.path(), "store-paths", b"/nix/store/compiler\n/nix/store/runtime\n");
    let btf = write_file(root.path(), "vmlinux", b"bounded BTF observation bytes");
    let headers = write_file(root.path(), "kernel-Makefile", b"VERSION = 6\n");
    let config = write_file(root.path(), "kernel.config", b"CONFIG_BPF=y\n");
    let generated_root = root.path().join("generated");
    fs::create_dir(&generated_root).unwrap();
    fs::write(generated_root.join("demo.c"), b"int main(void) { return 0; }\n").unwrap();
    fs::write(generated_root.join("demo.ebpf.c"), b"int probe(void *ctx) { return ctx != 0; }\n").unwrap();
    fs::write(generated_root.join("Makefile"), b"all:\n\tclang demo.ebpf.c\n").unwrap();
    if include_module {
        fs::write(generated_root.join("demo.mod.c"), b"int module(void) { return 0; }\n").unwrap();
        fs::write(generated_root.join("Kbuild"), b"obj-m += demo.mod.o\n").unwrap();
    }
    let archive_bytes = fs::read(&archive).unwrap();
    let archive_blake3 = blake3::hash(&archive_bytes).to_hex().to_string();
    let archive_sha256 = format!("{:x}", Sha256::digest(&archive_bytes));
    let mut request = request(source, archive, compiler, closure, btf, headers, config, archive_sha256, archive_blake3);
    if include_module {
        request.output_classes.push(ArtifactOutputClass::KernelModule);
        request.expected_generated_files.push(expected("demo.mod.c", GeneratedFileClass::ModuleC));
        request.expected_generated_files.push(expected("Kbuild", GeneratedFileClass::KbuildEvidence));
        request.toolchain.push(tool(ToolRole::KernelBuild, &request.compiler.executable_path));
        request.receipt_blockers.push(ExperimentBlocker {
            code: String::from(MODULE_BLOCKER_CODE),
            subject: String::from("kernel-module"),
            message: String::from("private/kfunc module authority remains externally blocked"),
        });
    }
    Fixture {
        _root: root,
        generated_root,
        request,
    }
}

#[allow(clippy::too_many_arguments)]
fn request(
    source: String,
    archive: String,
    compiler: String,
    closure: String,
    btf: String,
    headers: String,
    config: String,
    archive_sha256: String,
    archive_blake3: String,
) -> CoreAdapterRequest {
    CoreAdapterRequest {
        schema: String::from(ADAPTER_REQUEST_SCHEMA),
        experiment_id: String::from("adapter-parity-fixture"),
        source: SourceRequest {
            relative_path: String::from("demo.ks"),
            path: source,
        },
        compiler: CompilerRequest {
            version: String::from("0.1.2"),
            source_revision: String::from(SOURCE_REVISION),
            source_archive_url: String::from(SOURCE_URL),
            source_archive_path: archive,
            source_archive_sha256: archive_sha256,
            source_archive_blake3: archive_blake3,
            executable_path: compiler.clone(),
            closure_path_set_path: closure,
            closure_package: String::from("locked-nixpkgs-compiler-closure"),
            closure_version: String::from("fixture-only"),
        },
        toolchain: toolchain(&compiler),
        target: TargetRequest {
            architecture: KernelArchitecture::X86_64,
            kernel_release: String::from(KERNEL_RELEASE),
            cohort_label: String::from("locked-nixpkgs-observation-fixture"),
            btf_path: btf,
            headers_marker_path: headers,
            config_path: config,
        },
        output_classes: vec![
            ArtifactOutputClass::GeneratedSourceBundle,
            ArtifactOutputClass::UserspaceLoader,
            ArtifactOutputClass::EbpfObject,
        ],
        expected_generated_files: vec![
            expected("demo.c", GeneratedFileClass::UserspaceC),
            expected("demo.ebpf.c", GeneratedFileClass::EbpfC),
            expected("Makefile", GeneratedFileClass::MakefileEvidence),
        ],
        bpf_compiler_flags: vec![String::from("-target"), String::from("bpf"), String::from("-O2")],
        userspace_compiler_flags: vec![String::from("-O2"), String::from("-lbpf")],
        module_compiler_flags: vec![String::from("-Werror")],
        bounds: bounds(),
        receipt_blockers: Vec::new(),
    }
}

fn toolchain(path: &str) -> Vec<ToolRequest> {
    vec![
        tool(ToolRole::Ocaml, path),
        tool(ToolRole::Dune, path),
        tool(ToolRole::Menhir, path),
        tool(ToolRole::Clang, path),
        tool(ToolRole::CCompiler, path),
        tool(ToolRole::Bpftool, path),
        tool(ToolRole::Libbpf, path),
        tool(ToolRole::ElfLibrary, path),
        tool(ToolRole::Zlib, path),
    ]
}

fn tool(role: ToolRole, path: &str) -> ToolRequest {
    ToolRequest {
        role,
        version: String::from("fixture-only"),
        path: String::from(path),
        configuration: format!("fixture:{role:?}"),
    }
}

fn expected(path: &str, class: GeneratedFileClass) -> ExpectedGeneratedFile {
    ExpectedGeneratedFile {
        relative_path: String::from(path),
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

fn observed_fixture_files(root: &Path) -> Vec<ObservedGeneratedFile> {
    let mut files = fs::read_dir(root)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            ObservedGeneratedFile {
                relative_path: path.file_name().unwrap().to_str().unwrap().to_string(),
                bytes: fs::read(path).unwrap(),
                executable: false,
            }
        })
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    files
}

fn write_file(root: &Path, name: &str, bytes: &[u8]) -> String {
    let path = root.join(name);
    fs::write(&path, bytes).unwrap();
    path.to_str().unwrap().to_string()
}
