const VALID_DIGEST: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const UPPERCASE_DIGEST: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const CLI_PARSE_TEST_STACK_BYTES: usize = 16_777_216;
#[cfg(target_os = "linux")]
const EXECUTABLE_MODE: u32 = 0o755;
const TEST_CRT_FILES: &[&str] = &["Scrt1.o", "crti.o", "crtn.o", "ld-linux-x86-64.so.2", "libc.so"];
const TEST_LIBGCC_FILES: &[&str] = &["crtbeginS.o", "crtendS.o", "libgcc.a", "libgcc_eh.a"];

fn write_runtime_input_dir(path: &std::path::Path, names: &[&str]) {
    std::fs::create_dir(path).unwrap();
    for name in names {
        std::fs::write(path.join(name), b"runtime-input").unwrap();
    }
    assert!(path.is_dir());
    assert!(names.iter().all(|name| path.join(name).is_file()));
}

fn parse_cli(arguments: Vec<&'static str>) -> Result<crate::Args, String> {
    assert!(!arguments.is_empty());
    std::thread::Builder::new()
        .stack_size(CLI_PARSE_TEST_STACK_BYTES)
        .spawn(move || <crate::Args as clap::Parser>::try_parse_from(arguments).map_err(|error| error.to_string()))
        .map_err(|error| format!("starting Radiance CLI parser thread: {error}"))?
        .join()
        .map_err(|_| "Radiance CLI parser thread panicked".to_string())?
}

fn read_fixture(relative_path: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    let bytes = std::fs::read(&path).unwrap();
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    bytes
}

#[test]
fn embedded_profile_binds_the_exact_three_source_cohort() {
    let profile = crate::radiance::profile::load().unwrap();
    assert_eq!(profile.sources.len(), crunch_radiance_reference_core::RADIANCE_SOURCE_COUNT);
    assert_eq!(profile.sources[0].revision, crunch_radiance_reference_core::RADIANCE_REVISION);
    assert_eq!(
        profile.expected_fixed_point_blake3,
        "a06905539bd81c9581218ca648e98fb7a55c5a7e79847b6c840181a3e2605b07"
    );
    assert!(profile.sources.iter().all(|source| !source.repository_url.contains('@')));
}

#[test]
fn proof_request_requires_absent_output_and_explicit_executables() {
    let temp = tempfile::tempdir().unwrap();
    let tool = temp.path().join("tool");
    std::fs::write(&tool, b"tool").unwrap();
    let absent = temp.path().join("absent");
    let crt_dir = temp.path().join("crt");
    let libgcc_dir = temp.path().join("libgcc");
    write_runtime_input_dir(&crt_dir, TEST_CRT_FILES);
    write_runtime_input_dir(&libgcc_dir, TEST_LIBGCC_FILES);
    let request = crate::radiance::workflow::ProofRequest {
        source_bundle: &tool,
        expected_source_bundle_blake3: VALID_DIGEST,
        cohort: &tool,
        expected_cohort_blake3: VALID_DIGEST,
        cc: &tool,
        cc_driver: &tool,
        linker: &tool,
        crt_dir: &crt_dir,
        libgcc_dir: &libgcc_dir,
        output: &absent,
    };
    assert!(crate::radiance::workflow::validate_request(&request).is_ok());
    std::fs::remove_file(libgcc_dir.join("libgcc_eh.a")).unwrap();
    assert!(crate::radiance::workflow::validate_request(&request).is_err());
    std::fs::write(libgcc_dir.join("libgcc_eh.a"), b"runtime-input").unwrap();
    std::fs::create_dir(&absent).unwrap();
    assert!(crate::radiance::workflow::validate_request(&request).is_err());
}

#[test]
fn digest_and_role_admission_reject_malformed_values() {
    assert!(crate::radiance::workflow::helpers::valid_blake3_hex(VALID_DIGEST));
    assert!(!crate::radiance::workflow::helpers::valid_blake3_hex(UPPERCASE_DIGEST));
    assert!(!crate::radiance::workflow::helpers::valid_blake3_hex("short"));
    assert!(crate::radiance::source::role_from_label("radiance").is_ok());
    assert!(crate::radiance::source::role_from_label("ambient").is_err());
}

#[cfg(target_os = "linux")]
#[test]
fn checkout_admission_accepts_sha256_and_rejects_sha1() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let checkout = temp.path().join("checkout");
    std::fs::create_dir(&checkout).unwrap();
    std::fs::write(checkout.join("LICENSE"), include_bytes!("../../fixtures/radiance-reference/LICENSE.MIT")).unwrap();
    let git = temp.path().join("git");
    let valid_script = format!(
        "#!/bin/sh\ncase \"$*\" in\n  'rev-parse --show-object-format') printf 'sha256\\n' ;;\n  'rev-parse HEAD') printf '{}\\n' ;;\n  'status --porcelain=v1 --untracked-files=all') exit 0 ;;\n  *) exit 9 ;;\nesac\n",
        crunch_radiance_reference_core::RADIANCE_REVISION
    );
    std::fs::write(&git, &valid_script).unwrap();
    std::fs::set_permissions(&git, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).unwrap();
    assert!(
        crate::radiance::source::validate_checkout(
            &git,
            &checkout,
            crunch_radiance_reference_core::RadianceSourceRole::Radiance
        )
        .is_ok()
    );
    let sha1_script = valid_script.replace("sha256", "sha1");
    std::fs::write(&git, sha1_script).unwrap();
    assert!(
        crate::radiance::source::validate_checkout(
            &git,
            &checkout,
            crunch_radiance_reference_core::RadianceSourceRole::Radiance
        )
        .is_err()
    );
}

#[test]
fn protected_audit_matches_receipt_tools_and_rejects_digest_drift() {
    let receipt_bytes = read_fixture("schemas/machine-contracts/fixtures/radiance-reference-receipt.valid.json");
    let receipt: crunch_radiance_reference_core::RadianceReferenceReceipt =
        serde_json::from_slice(&receipt_bytes).unwrap();
    let audit = read_fixture("fixtures/radiance-reference/protected-execution-audit.valid.json");
    assert!(crate::radiance::publication::validate_protected_audit_bytes(&audit, &receipt).is_ok());

    let mut tampered: serde_json::Value = serde_json::from_slice(&audit).unwrap();
    tampered["events"][0]["digest_hex"] = VALID_DIGEST.into();
    let tampered = crate::radiance::source::canonical_pretty_json(&tampered, "tampered Radiance audit").unwrap();
    assert!(crate::radiance::publication::validate_protected_audit_bytes(&tampered, &receipt).is_err());
}

#[test]
fn cli_requires_the_explicit_compiler_driver_for_proof() {
    let valid = parse_cli(vec![
        "mantle",
        "bootstrap",
        "radiance-reference",
        "prove",
        "--source-bundle",
        "/source.json",
        "--expected-source-bundle-blake3",
        VALID_DIGEST,
        "--cohort",
        "/cohort.json",
        "--expected-cohort-blake3",
        VALID_DIGEST,
        "--cc",
        "/cc-wrapper",
        "--cc-driver",
        "/cc-driver",
        "--linker",
        "/linker",
        "--crt-dir",
        "/crt",
        "--libgcc-dir",
        "/libgcc",
        "--output",
        "/proof",
    ]);
    let missing_driver = parse_cli(vec![
        "mantle",
        "bootstrap",
        "radiance-reference",
        "prove",
        "--source-bundle",
        "/source.json",
        "--expected-source-bundle-blake3",
        VALID_DIGEST,
        "--cohort",
        "/cohort.json",
        "--expected-cohort-blake3",
        VALID_DIGEST,
        "--cc",
        "/cc-wrapper",
        "--linker",
        "/linker",
        "--crt-dir",
        "/crt",
        "--libgcc-dir",
        "/libgcc",
        "--output",
        "/proof",
    ]);
    assert!(valid.is_ok());
    assert!(missing_driver.is_err());
}
