//! Integration tests: generated inputs.ncl imports cleanly from Nickel.

use std::collections::BTreeMap;

use crunch_project::*;

fn lock_with_entries() -> Lockfile {
    let mut inputs = BTreeMap::new();
    inputs.insert("nixpkgs".to_string(), LockEntry {
        kind: LockedKind::Git {
            repository: "https://github.com/NixOS/nixpkgs.git".to_string(),
            rev: "abc123def456".to_string(),
            ref_name: Some("nixos-unstable".to_string()),
        },
        hash: LockedHash {
            algo: HashAlgo::Sha256,
            value: "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string(),
        },
        patches: vec![],
        mirrors: vec!["https://mirrors.tuna.tsinghua.edu.cn/git/nixpkgs.git".to_string()],
        fetch_policy: InputFetchPolicy::GenerationMaterial,
    });
    inputs.insert("hello-src".to_string(), LockEntry {
        kind: LockedKind::Tarball {
            url: "https://ftp.gnu.org/gnu/hello/hello-2.12.1.tar.gz".to_string(),
        },
        hash: LockedHash {
            algo: HashAlgo::Sha256,
            value: "sha256-jZkUKv2SV28wsM18tCqNxoCZmLxdYH2Idh9RLibH2yA=".to_string(),
        },
        patches: vec![],
        mirrors: vec![],
        fetch_policy: InputFetchPolicy::GenerationMaterial,
    });
    Lockfile {
        version: SchemaVersion::CURRENT,
        inputs,
        patches: BTreeMap::new(),
    }
}

#[test]
fn generated_inputs_is_valid_nickel() {
    let lock = lock_with_entries();
    let ncl = generate_inputs_ncl(lock);

    // Write to temp file and evaluate with crunch-eval
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("inputs.ncl");
    std::fs::write(&file, &ncl).unwrap();

    let expr = crunch_eval::evaluate(&file, &[]).unwrap();
    assert!(expr.is_record(), "generated file should evaluate to a record");

    let record = expr.as_record().unwrap();
    // Check that both entries exist
    let nixpkgs = record.value_by_name("nixpkgs").expect("missing nixpkgs");
    assert!(nixpkgs.is_record());

    let hello = record.value_by_name("hello-src").expect("missing hello-src");
    assert!(hello.is_record());
}

#[test]
fn generated_inputs_field_values_match_lock() {
    let lock = lock_with_entries();
    let ncl = generate_inputs_ncl(lock);

    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("inputs.ncl");
    std::fs::write(&file, &ncl).unwrap();

    let expr = crunch_eval::evaluate(&file, &[]).unwrap();
    let record = expr.as_record().unwrap();

    // Check nixpkgs fields
    let nixpkgs = record.value_by_name("nixpkgs").unwrap();
    let nix_rec = nixpkgs.as_record().unwrap();
    assert_eq!(nix_rec.value_by_name("type").unwrap().as_str(), Some("git"));
    assert_eq!(nix_rec.value_by_name("rev").unwrap().as_str(), Some("abc123def456"));
    assert_eq!(nix_rec.value_by_name("ref_name").unwrap().as_str(), Some("nixos-unstable"));
    assert_eq!(
        nix_rec.value_by_name("hash").unwrap().as_str(),
        Some("sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")
    );

    // Check mirrors array
    let mirrors = nix_rec.value_by_name("mirrors").unwrap();
    let mirrors_arr = mirrors.as_array().unwrap();
    assert_eq!(mirrors_arr.len(), 1);

    // Check hello-src fields
    let hello = record.value_by_name("hello-src").unwrap();
    let hello_rec = hello.as_record().unwrap();
    assert_eq!(hello_rec.value_by_name("type").unwrap().as_str(), Some("tarball"));
    assert_eq!(
        hello_rec.value_by_name("url").unwrap().as_str(),
        Some("https://ftp.gnu.org/gnu/hello/hello-2.12.1.tar.gz")
    );
}

#[test]
fn generated_inputs_importable_from_package_code() {
    let lock = lock_with_entries();
    let ncl = generate_inputs_ncl(lock);

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("inputs.ncl"), &ncl).unwrap();

    // Write a "package" file that imports the generated inputs
    let package_ncl = r#"
        let inputs = import "inputs.ncl" in
        {
            nixpkgs_rev = inputs.nixpkgs.rev,
            hello_url = inputs."hello-src".url,
        }
    "#;
    let package_file = dir.path().join("package.ncl");
    std::fs::write(&package_file, package_ncl).unwrap();

    let expr = crunch_eval::evaluate(&package_file, &[]).unwrap();
    let record = expr.as_record().unwrap();

    assert_eq!(record.value_by_name("nixpkgs_rev").unwrap().as_str(), Some("abc123def456"));
    assert_eq!(
        record.value_by_name("hello_url").unwrap().as_str(),
        Some("https://ftp.gnu.org/gnu/hello/hello-2.12.1.tar.gz")
    );
}

#[test]
fn drift_detection_matches_generation() {
    let lock = lock_with_entries();
    let ncl = generate_inputs_ncl(lock.clone());

    // Fresh generation is in sync
    assert!(check_drift(lock.clone(), Some(ncl)).is_ok());

    // Missing file
    assert_eq!(check_drift(lock.clone(), None), DriftStatus::Missing);

    // Stale content
    assert!(matches!(check_drift(lock, Some("old content".to_string())), DriftStatus::Drifted { .. }));
}

#[test]
fn malformed_manifest_loading_via_nickel_eval_fails() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_ncl = r#"
        {
            inputs = [],
            patches = [],
        }
    "#;
    let file = dir.path().join("manifest.ncl");
    std::fs::write(&file, manifest_ncl).unwrap();

    let result = crunch_eval::evaluate_and_deserialize::<ProjectManifest>(&file, &[]);

    assert!(result.is_err());
}

#[test]
fn manifest_loading_via_nickel_eval() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_ncl = r#"
        {
            version = "1.0.0",
            inputs = [
                {
                    name = "mylib",
                    kind = { type = "tarball", url = "https://example.com/mylib.tar.gz" },
                },
            ],
            patches = [],
        }
    "#;
    let file = dir.path().join("manifest.ncl");
    std::fs::write(&file, manifest_ncl).unwrap();

    let manifest: ProjectManifest = crunch_eval::evaluate_and_deserialize(&file, &[]).unwrap();

    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.inputs.len(), 1);
    assert_eq!(manifest.inputs[0].name, "mylib");
    match &manifest.inputs[0].kind {
        InputKind::Tarball { url } => {
            assert_eq!(url, "https://example.com/mylib.tar.gz");
        }
        other => panic!("expected Tarball, got {other:?}"),
    }
}
