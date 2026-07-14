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
