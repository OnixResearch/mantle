mod support;

use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::process::Output;

use assert_cmd::Command;
use mantle::oci_projection::OCI_EXPORT_REPORT_FILENAME;
use mantle::oci_projection::OciImportReport;
use mantle::oci_registry::OciRegistryPullReport;
use mantle::oci_registry::OciRegistryPushReport;
use mantle::oci_registry::verify_pull_report;
use mantle::oci_registry::verify_push_report;
use support::oci_registry::TestRegistry;
use tempfile::TempDir;

const EXAMPLE_ROOT: &str = "examples/projects/kernel-bundle-oci-local";
const KERNEL_RELATIVE_PATH: &str = "fixtures/vmlinuz";
const MODULES_RELATIVE_PATH: &str = "fixtures/modules";
const SPEC_RELATIVE_PATH: &str = "fixtures/kernel-bundles-spec.md";
const PROJECTION_RELATIVE_PATH: &str = "fixtures/projection.json";
const SOURCE_ADMISSIONS_RELATIVE_PATH: &str = "fixtures/source-admissions.json";
const REPOSITORY: &str = "onix/kernel-bundle";
const REFERENCE: &str = "gallery";
const BEARER_TOKEN: &str = "gallery-registry-token";

struct PublishedFixture {
    _root: TempDir,
    registry: TestRegistry,
    token_file: PathBuf,
    source_layout: PathBuf,
    push_report: OciRegistryPushReport,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn example_root() -> PathBuf {
    repo_root().join(EXAMPLE_ROOT)
}

fn mantle_cmd() -> Command {
    Command::cargo_bin("mantle").expect("mantle binary should be built")
}

fn run(command: &mut Command, operation: &str) -> Output {
    let output = command.output().unwrap_or_else(|error| panic!("{operation} should run: {error}"));
    if !output.status.success() {
        panic!("{operation} failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    output
}

fn import_gallery_objects(root: &Path, state: &Path) {
    let example = example_root();
    for (source, report_name) in [
        (example.join(KERNEL_RELATIVE_PATH), "kernel-import.json"),
        (example.join(MODULES_RELATIVE_PATH), "modules-import.json"),
    ] {
        run(
            mantle_cmd()
                .arg("--json")
                .arg("--state-dir")
                .arg(state)
                .args(["artifact", "import"])
                .arg(source)
                .arg("--report-out")
                .arg(root.join(report_name)),
            "gallery artifact import",
        );
    }
    assert!(state.is_dir(), "gallery source state must exist after imports");
    assert!(root.join("kernel-import.json").is_file(), "kernel admission report must exist");
}

fn export_gallery_layout(root: &Path) -> PathBuf {
    let example = example_root();
    let source_state = root.join("source-state");
    let layout = root.join("source-layout");
    import_gallery_objects(root, &source_state);
    run(
        mantle_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&source_state)
            .args(["artifact", "oci-export", "--projection"])
            .arg(example.join(PROJECTION_RELATIVE_PATH))
            .arg("--spec-material")
            .arg(example.join(SPEC_RELATIVE_PATH))
            .arg("--source-admissions")
            .arg(example.join(SOURCE_ADMISSIONS_RELATIVE_PATH))
            .arg("--out")
            .arg(&layout),
        "gallery OCI export",
    );
    assert!(layout.join("index.json").is_file(), "gallery OCI index must exist");
    assert!(layout.join(OCI_EXPORT_REPORT_FILENAME).is_file(), "gallery OCI export report must exist");
    layout
}

fn push_command(layout: &Path, registry: &TestRegistry, token_file: Option<&Path>, receipt_out: &Path) -> Command {
    let mut command = mantle_cmd();
    command
        .arg("--json")
        .args(["artifact", "oci-push", "--layout"])
        .arg(layout)
        .arg("--registry")
        .arg(registry.url())
        .arg("--repository")
        .arg(REPOSITORY)
        .arg("--reference")
        .arg(REFERENCE)
        .arg("--allow-http")
        .arg("--receipt-out")
        .arg(receipt_out);
    if let Some(token_file) = token_file {
        command.arg("--bearer-token-file").arg(token_file);
    }
    command
}

fn pull_command(
    fixture: &PublishedFixture,
    output_layout: &Path,
    state: &Path,
    import_report: &Path,
    pull_receipt: &Path,
) -> Command {
    let mut command = mantle_cmd();
    command
        .arg("--json")
        .arg("--state-dir")
        .arg(state)
        .args(["artifact", "oci-pull", "--registry"])
        .arg(fixture.registry.url())
        .arg("--repository")
        .arg(REPOSITORY)
        .arg("--reference")
        .arg(REFERENCE)
        .arg("--expected-manifest-digest")
        .arg(&fixture.push_report.manifest_digest)
        .arg("--expected-metadata-manifest-digest")
        .arg(&fixture.push_report.metadata_manifest_digest)
        .arg("--bearer-token-file")
        .arg(&fixture.token_file)
        .arg("--allow-http")
        .arg("--out")
        .arg(output_layout)
        .arg("--report-out")
        .arg(import_report)
        .arg("--receipt-out")
        .arg(pull_receipt);
    command
}

fn publish_fixture() -> PublishedFixture {
    let root = TempDir::new().expect("temporary registry fixture should exist");
    let source_layout = export_gallery_layout(root.path());
    let registry = TestRegistry::start(BEARER_TOKEN);
    let token_file = root.path().join("registry-token");
    let push_receipt = root.path().join("push-receipt.json");
    registry.write_token_file(&token_file);
    let output = run(&mut push_command(&source_layout, &registry, Some(&token_file), &push_receipt), "registry push");
    let report: OciRegistryPushReport = serde_json::from_slice(&output.stdout).expect("push receipt should parse");
    verify_push_report(&report).expect("push receipt should verify");
    assert!(push_receipt.is_file(), "push receipt file must be published");
    assert!(!String::from_utf8_lossy(&output.stdout).contains(BEARER_TOKEN));
    PublishedFixture {
        _root: root,
        registry,
        token_file,
        source_layout,
        push_report: report,
    }
}

fn collect_layout_files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, path: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        let mut entries = std::fs::read_dir(path)
            .unwrap_or_else(|error| panic!("reading {}: {error}", path.display()))
            .map(|entry| entry.expect("layout entry should read").path())
            .collect::<Vec<_>>();
        entries.sort();
        for entry in entries {
            let metadata = std::fs::symlink_metadata(&entry).expect("layout metadata should read");
            assert!(!metadata.file_type().is_symlink(), "gallery layout must not contain symlinks");
            if metadata.is_dir() {
                walk(root, &entry, files);
            } else {
                let relative = entry.strip_prefix(root).expect("layout path should remain below root");
                files.insert(relative.to_string_lossy().replace('\\', "/"), std::fs::read(&entry).unwrap());
            }
        }
    }
    let mut files = BTreeMap::new();
    walk(root, root, &mut files);
    assert!(!files.is_empty(), "layout file map must not be empty");
    assert!(files.contains_key("index.json"), "layout file map must contain the OCI index");
    files
}

fn assert_failed_pull_has_no_outputs(root: &Path) {
    for name in ["pulled-layout", "pull-state", "import-report.json", "pull-receipt.json"] {
        assert!(!root.join(name).exists(), "failed pull unexpectedly created {name}");
    }
}

#[test]
fn registry_cli_pushes_and_pulls_exact_admitted_layout_into_fresh_state() {
    let fixture = publish_fixture();
    let root = fixture._root.path();
    let pulled_layout = root.join("pulled-layout");
    let state = root.join("pull-state");
    let import_report_path = root.join("import-report.json");
    let pull_receipt_path = root.join("pull-receipt.json");
    let output = run(
        &mut pull_command(&fixture, &pulled_layout, &state, &import_report_path, &pull_receipt_path),
        "registry pull",
    );
    let pull: OciRegistryPullReport = serde_json::from_slice(&output.stdout).expect("pull receipt should parse");
    verify_pull_report(&pull).expect("pull receipt should verify");
    let imported: OciImportReport = serde_json::from_slice(&std::fs::read(&import_report_path).unwrap()).unwrap();
    assert_eq!(pull.expected_manifest_digest, fixture.push_report.manifest_digest);
    assert_eq!(pull.expected_metadata_manifest_digest, fixture.push_report.metadata_manifest_digest);
    assert_eq!(pull.import_receipt_blake3, imported.receipt_blake3);
    assert_eq!(imported.state, "admitted");
    assert_eq!(collect_layout_files(&fixture.source_layout), collect_layout_files(&pulled_layout));
    assert!(state.is_dir(), "fresh pull state must contain admitted objects");
    assert!(pull_receipt_path.is_file(), "pull receipt file must be published");
    assert!(!String::from_utf8_lossy(&output.stdout).contains(BEARER_TOKEN));
}

#[test]
fn registry_pull_rejects_mutable_image_tag_before_layout_or_admission() {
    let fixture = publish_fixture();
    let root = fixture._root.path();
    fixture.registry.replace_tag_with_drift(REPOSITORY, REFERENCE);
    let output = pull_command(
        &fixture,
        &root.join("pulled-layout"),
        &root.join("pull-state"),
        &root.join("import-report.json"),
        &root.join("pull-receipt.json"),
    )
    .output()
    .expect("drifted pull should run");
    assert!(!output.status.success(), "drifted image tag must fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unexpected digest"), "stderr: {stderr}");
    assert_failed_pull_has_no_outputs(root);
}

#[test]
fn registry_pull_rejects_tampered_metadata_blob_before_layout_or_admission() {
    let fixture = publish_fixture();
    let root = fixture._root.path();
    fixture.registry.tamper_metadata_blob(REPOSITORY, &fixture.push_report.metadata_manifest_digest);
    let output = pull_command(
        &fixture,
        &root.join("pulled-layout"),
        &root.join("pull-state"),
        &root.join("import-report.json"),
        &root.join("pull-receipt.json"),
    )
    .output()
    .expect("tampered metadata pull should run");
    assert!(!output.status.success(), "tampered metadata blob must fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("do not match their descriptor"), "stderr: {stderr}");
    assert_failed_pull_has_no_outputs(root);
}

#[test]
fn registry_push_requires_credentials_and_reuses_blobs_after_interruption() {
    let root = TempDir::new().expect("temporary registry fixture should exist");
    let layout = export_gallery_layout(root.path());
    let registry = TestRegistry::start(BEARER_TOKEN);
    let missing_receipt = root.path().join("missing-credential-receipt.json");
    let denied = push_command(&layout, &registry, None, &missing_receipt)
        .output()
        .expect("anonymous push should run");
    assert!(!denied.status.success(), "missing registry credentials must fail");
    let denied_stderr = String::from_utf8_lossy(&denied.stderr);
    assert!(denied_stderr.contains("HTTP 401"), "stderr: {denied_stderr}");
    assert!(!missing_receipt.exists(), "denied push must not write a receipt");

    let token_file = root.path().join("registry-token");
    registry.write_token_file(&token_file);
    registry.fail_next_manifest(REFERENCE);
    let interrupted_receipt = root.path().join("interrupted-receipt.json");
    let interrupted = push_command(&layout, &registry, Some(&token_file), &interrupted_receipt)
        .output()
        .expect("interrupted push should run");
    assert!(!interrupted.status.success(), "interrupted image publication must fail");
    let interrupted_stderr = String::from_utf8_lossy(&interrupted.stderr);
    assert!(interrupted_stderr.contains("HTTP 500"), "stderr: {interrupted_stderr}");
    assert!(!interrupted_receipt.exists(), "interrupted push must not write a receipt");

    let retry_receipt = root.path().join("retry-receipt.json");
    let retry = run(&mut push_command(&layout, &registry, Some(&token_file), &retry_receipt), "registry push retry");
    let report: OciRegistryPushReport = serde_json::from_slice(&retry.stdout).expect("retry receipt should parse");
    verify_push_report(&report).expect("retry receipt should verify");
    assert_eq!(report.uploaded_blobs, 0, "retry should not duplicate blob writes");
    assert!(report.reused_blobs > 0, "retry should reuse previously verified blobs");
    assert!(retry_receipt.is_file(), "successful retry must publish a receipt");
    assert!(!String::from_utf8_lossy(&retry.stdout).contains(BEARER_TOKEN));
}
