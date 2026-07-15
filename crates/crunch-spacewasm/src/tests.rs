use std::fs;
use std::path::PathBuf;

use tempfile::tempdir;

use super::*;

fn request() -> MaterializationRequest {
    MaterializationRequest {
        schema: String::from(MATERIALIZATION_REQUEST_SCHEMA),
        profile_path: String::from("profile.json"),
        source_facts_path: String::from("source-facts.json"),
        support_matrix_path: String::from("support.json"),
        checks_path: String::from("checks.json"),
        requested_claim_class: String::from(REFERENCE_CLAIM_CLASS),
        members: vec![InputMember {
            source_path: String::from("source"),
            bundle_path: String::from("profile/profile.ncl"),
            role: BundleRole::ProfileSource,
        }],
    }
}

#[test]
fn request_reader_accepts_regular_json_and_rejects_symlink() {
    let temp = tempdir().unwrap();
    let request_path = temp.path().join("request.json");
    fs::write(&request_path, serde_json::to_vec(&request()).unwrap()).unwrap();
    let loaded = read_materialization_request(&request_path).unwrap();
    assert_eq!(loaded.schema, MATERIALIZATION_REQUEST_SCHEMA);
    assert_eq!(loaded.members.len(), 1);

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let symlink_path = temp.path().join("request-link.json");
        symlink(&request_path, &symlink_path).unwrap();
        let error = read_materialization_request(&symlink_path).unwrap_err();
        assert!(error.to_string().contains("opening no-follow input"));
        assert!(!error.to_string().is_empty());
    }
}

#[test]
fn materialization_rejects_preexisting_output_without_mutation() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("bundle");
    fs::create_dir(&output).unwrap();
    fs::write(output.join("sentinel"), b"keep").unwrap();

    let error = materialize(request(), &output).unwrap_err();

    assert!(error.to_string().contains("already exists"));
    assert_eq!(fs::read(output.join("sentinel")).unwrap(), b"keep");
}

#[test]
fn verify_rejects_symlinked_bundle_root() {
    let temp = tempdir().unwrap();
    let real = temp.path().join("real");
    fs::create_dir(&real).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let link = temp.path().join("bundle-link");
        symlink(&real, &link).unwrap();
        let error = verify_bundle(&link).unwrap_err();
        assert!(error.to_string().contains("not a real directory"));
        assert!(PathBuf::from(&link).exists());
    }
}

#[test]
fn failed_materialization_removes_owned_output() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("bundle");
    let mut invalid_request = request();
    invalid_request.schema = String::from("unsupported-schema");

    let error = materialize(invalid_request, &output).unwrap_err();

    assert!(error.to_string().contains("unsupported request schema"));
    assert!(!output.exists());
}

#[test]
fn cleanup_failure_preserves_materialization_error() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("not-a-directory");
    fs::write(&output, b"occupied").unwrap();
    let materialization_error = ShellError::Invalid(String::from("original materialization failure"));

    let cleanup_error = cleanup_failed_materialization(&output, materialization_error);
    let message = cleanup_error.to_string();

    assert!(message.contains("original materialization failure"));
    assert!(message.contains("removing incomplete bundle"));
    assert!(output.is_file());
}

#[test]
fn request_header_rejects_empty_and_oversized_member_sets() {
    let mut empty_request = request();
    empty_request.members.clear();
    let empty_error = validate_request_header(&empty_request).unwrap_err();
    assert!(empty_error.to_string().contains("member count"));
    assert!(empty_request.members.is_empty());

    let mut oversized_request = request();
    let member = oversized_request.members[0].clone();
    let member_slots = usize::try_from(HARD_MAX_BUNDLE_FILES).unwrap();
    oversized_request.members = vec![member; member_slots + 1];
    let oversized_error = validate_request_header(&oversized_request).unwrap_err();
    assert!(oversized_error.to_string().contains("member count"));
    assert!(oversized_request.members.len() > member_slots);
}

#[test]
fn bounded_reader_accepts_limit_and_rejects_invalid_sizes() {
    const OVERSIZED_INPUT_BYTES: usize = 2;

    let temp = tempdir().unwrap();
    let bounded_path = temp.path().join("bounded");
    fs::write(&bounded_path, b"x").unwrap();
    assert_eq!(read_regular_file_bounded(&bounded_path, 1).unwrap(), b"x");

    let empty_path = temp.path().join("empty");
    fs::write(&empty_path, b"").unwrap();
    let empty_error = read_regular_file_bounded(&empty_path, 1).unwrap_err();
    assert!(empty_error.to_string().contains("empty or exceeds"));

    let oversized_path = temp.path().join("oversized");
    fs::write(&oversized_path, [b'x'; OVERSIZED_INPUT_BYTES]).unwrap();
    let oversized_error = read_regular_file_bounded(&oversized_path, 1).unwrap_err();
    assert!(oversized_error.to_string().contains("empty or exceeds"));
}

#[test]
fn bundle_file_collection_enforces_capacity() {
    const EXPECTED_FILE_COUNT: usize = 2;

    let temp = tempdir().unwrap();
    fs::write(temp.path().join("first"), b"first").unwrap();
    fs::write(temp.path().join("second"), b"second").unwrap();

    let files = collect_bundle_files(temp.path(), u32::try_from(EXPECTED_FILE_COUNT).unwrap()).unwrap();
    assert_eq!(files.len(), EXPECTED_FILE_COUNT);
    assert_eq!(files, vec![String::from("first"), String::from("second")]);

    let error = collect_bundle_files(temp.path(), 1).unwrap_err();
    assert!(error.to_string().contains("file count"));
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), EXPECTED_FILE_COUNT);
}
