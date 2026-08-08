#![cfg(unix)]

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use crunch_nar::CaseHackPolicy;
use crunch_nar::CutoverEvidence;
use crunch_nar::FilesystemNarError;
use crunch_nar::FilesystemNarRequest;
use crunch_nar::OptionalOracleDisposition;
use crunch_nar::ParityComparison;
use crunch_nar::UPSTREAM_CARGO_CHECKSUM;
use crunch_nar::UPSTREAM_COMMIT;
use crunch_nar::UPSTREAM_VERSION;
use crunch_nar::digest_bytes;
use crunch_nar::evaluate_cutover;
use crunch_nar::observe_path;
use nix_archive::nar;
use nix_compat::nixhash::HashAlgo;
use snix_castore::blobservice::MemoryBlobService;
use snix_castore::directoryservice::RedbDirectoryService;
use snix_castore::directoryservice::RedbDirectoryServiceConfig;
use snix_castore::import::fs::ingest_path;
use snix_store::nar::write_nar;
use snix_store::utils::AsyncIoBridge;
use tempfile::TempDir;

const FIXTURE_NAR_BYTES_MAX: u64 = 16_777_216;
const GENERATED_CASE_COUNT: u32 = 8;
const GENERATED_DEPTH_MAX: u32 = 4;
const GENERATED_FILE_BYTES_MAX: u32 = 256;
const GENERATED_ENTRY_COUNT_MAX: u32 = 64;
const DETERMINISTIC_SEED: u32 = 0x4E_41_52_31;
const OWNER_EXECUTABLE_MODE: u32 = 0o700;
const OWNER_READ_WRITE_MODE: u32 = 0o600;
const NO_PERMISSIONS_MODE: u32 = 0o000;
const NON_UTF8_NAME: &[u8] = b"weird-\xff\xfe-name";
const NON_UTF8_TARGET: &[u8] = b"target-\xff-bytes";
const REGULAR_GOLDEN: &str = include_str!("../fixtures/upstream/regular.nar.hex");
const EXECUTABLE_GOLDEN: &str = include_str!("../fixtures/upstream/executable.nar.hex");
const SYMLINK_GOLDEN: &str = include_str!("../fixtures/upstream/symlink.nar.hex");
const DIRECTORY_GOLDEN: &str = include_str!("../fixtures/upstream/directory.nar.hex");
const FIXTURE_MANIFEST: &str = include_str!("../fixtures/upstream/manifest.ncl");
const REGULAR_FIXTURE_BLAKE3: &str = "e957e7172e05065e35156c6626f6e9a40e190f427edb9be33c5802e3fd4b699f";
const EXECUTABLE_FIXTURE_BLAKE3: &str = "9e4897bc6af84f92dcb504adb4ef27afd385e2df5042a5406263741fc9196dc1";
const SYMLINK_FIXTURE_BLAKE3: &str = "9d042bf21abd657aca6d483b02aa5e2a418530c9b5b244c71f8b59f5bfeafd95";
const DIRECTORY_FIXTURE_BLAKE3: &str = "e4c0d9801bc407d8360843bd825398b8c9560b60ca2f4f4f35b054a21f61d651";

struct Fixture {
    _temporary: TempDir,
    root: PathBuf,
}

fn parity_fixture() -> Fixture {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().join("tree");
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir_all(root.join("sub/deeper")).unwrap();
    std::fs::create_dir_all(root.join("sub/empty-dir")).unwrap();
    std::fs::write(root.join("hello.txt"), b"hello world\n").unwrap();
    std::fs::write(root.join("empty"), b"").unwrap();
    std::fs::write(root.join("sub/deeper/file"), b"nested\n").unwrap();
    std::fs::write(root.join("run.sh"), b"#!/bin/sh\necho hi\n").unwrap();
    std::fs::set_permissions(root.join("run.sh"), std::fs::Permissions::from_mode(OWNER_EXECUTABLE_MODE)).unwrap();
    std::fs::write(root.join("a"), b"1").unwrap();
    std::fs::write(root.join("a-b"), b"2").unwrap();
    std::fs::write(root.join("ab"), b"3").unwrap();
    std::os::unix::fs::symlink("hello.txt", root.join("link")).unwrap();
    std::os::unix::fs::symlink("sub", root.join("linkdir")).unwrap();
    std::os::unix::fs::symlink(OsStr::from_bytes(NON_UTF8_TARGET), root.join("badlink")).unwrap();
    std::fs::write(root.join(OsStr::from_bytes(NON_UTF8_NAME)), b"attack of the bytes").unwrap();
    Fixture {
        _temporary: temporary,
        root,
    }
}

fn snix_nar(path: &Path) -> Vec<u8> {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let blob_service = MemoryBlobService::default();
    let directory_service =
        RedbDirectoryService::new_temporary("crunch-nar-parity".to_string(), RedbDirectoryServiceConfig::default())
            .unwrap();
    runtime.block_on(async move {
        let node = ingest_path::<_, _, _, &[u8]>(blob_service.clone(), directory_service.clone(), path, None)
            .await
            .unwrap();
        let mut bytes = Vec::new();
        write_nar(AsyncIoBridge(&mut bytes), &node, blob_service, directory_service).await.unwrap();
        bytes
    })
}

fn decode_hex(source: &str) -> Vec<u8> {
    let digits: Vec<u8> = source.bytes().filter(|byte| !byte.is_ascii_whitespace()).collect();
    assert_eq!(digits.len() % 2, 0, "fixture hex digit count must be even");
    assert!(!digits.is_empty(), "fixture hex must not be empty");
    digits
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16).unwrap();
            let low = (pair[1] as char).to_digit(16).unwrap();
            u8::try_from((high << 4) | low).unwrap()
        })
        .collect()
}

fn adapter_bytes(path: &Path) -> Vec<u8> {
    let mut bytes = Vec::new();
    nar::encode_path_with_case_hack(&mut bytes, path, nar::CaseHack::Disabled).unwrap();
    assert!(u64::try_from(bytes.len()).unwrap() <= FIXTURE_NAR_BYTES_MAX);
    assert!(!bytes.is_empty());
    bytes
}

fn all_algorithms() -> [HashAlgo; 5] {
    [
        HashAlgo::Md5,
        HashAlgo::Sha1,
        HashAlgo::Sha256,
        HashAlgo::Sha512,
        HashAlgo::Blake3,
    ]
}

// r[verify store_transports.nix_archive_parity]
// r[verify store_transports.nix_archive_filesystem_observation]
#[test]
fn nix_archive_and_snix_match_on_byte_safe_fixture_and_all_recursive_hashes() {
    let fixture = parity_fixture();
    let adapter = adapter_bytes(&fixture.root);
    let snix = snix_nar(&fixture.root);
    assert_eq!(adapter, snix);
    assert!(adapter.len() > NON_UTF8_NAME.len());

    for algorithm in all_algorithms() {
        let observation = observe_path(
            &fixture.root,
            FilesystemNarRequest::new(algorithm, CaseHackPolicy::Disabled, FIXTURE_NAR_BYTES_MAX),
        )
        .unwrap();
        assert_eq!(observation.nar_size, u64::try_from(snix.len()).unwrap());
        assert_eq!(observation.digest, digest_bytes(&snix, algorithm));
    }
}

// r[verify store_transports.nix_archive_parity]
#[test]
fn retained_fixture_manifest_binds_source_and_payload_identities() {
    let fixtures = [
        (REGULAR_GOLDEN, REGULAR_FIXTURE_BLAKE3),
        (EXECUTABLE_GOLDEN, EXECUTABLE_FIXTURE_BLAKE3),
        (SYMLINK_GOLDEN, SYMLINK_FIXTURE_BLAKE3),
        (DIRECTORY_GOLDEN, DIRECTORY_FIXTURE_BLAKE3),
    ];
    assert!(FIXTURE_MANIFEST.contains(UPSTREAM_COMMIT));
    assert!(FIXTURE_MANIFEST.contains(UPSTREAM_CARGO_CHECKSUM));
    for (fixture, expected_blake3) in fixtures {
        assert_eq!(blake3::hash(fixture.as_bytes()).to_hex().as_str(), expected_blake3);
        assert!(FIXTURE_MANIFEST.contains(expected_blake3));
    }
}

// r[verify store_transports.nix_archive_parity]
#[test]
fn retained_upstream_goldens_match_filesystem_encoding() {
    let temporary = tempfile::tempdir().unwrap();
    let regular = temporary.path().join("regular");
    std::fs::write(&regular, b"hi\n").unwrap();
    assert_eq!(adapter_bytes(&regular), decode_hex(REGULAR_GOLDEN));

    let executable = temporary.path().join("executable");
    std::fs::write(&executable, b"#!/bin/bash\n\ngcc -o hello hello.c\n").unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(OWNER_EXECUTABLE_MODE)).unwrap();
    assert_eq!(adapter_bytes(&executable), decode_hex(EXECUTABLE_GOLDEN));

    let symlink = temporary.path().join("symlink");
    std::os::unix::fs::symlink("hello.c", &symlink).unwrap();
    assert_eq!(adapter_bytes(&symlink), decode_hex(SYMLINK_GOLDEN));

    let directory = temporary.path().join("directory");
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(directory.join("build.sh"), b"#!/bin/bash\n\ngcc -o hello hello.c\n").unwrap();
    std::fs::set_permissions(directory.join("build.sh"), std::fs::Permissions::from_mode(OWNER_EXECUTABLE_MODE))
        .unwrap();
    std::fs::write(directory.join("hello.c"), b"#include <stdio.h>\n\nint main(int argc, char *argv[]){ exit 0; }\n")
        .unwrap();
    std::os::unix::fs::symlink("hello.c", directory.join("hi.c")).unwrap();
    assert_eq!(adapter_bytes(&directory), decode_hex(DIRECTORY_GOLDEN));
}

fn generated_tree(case_index: u32) -> Fixture {
    assert!(case_index < GENERATED_CASE_COUNT);
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().join("generated");
    std::fs::create_dir(&root).unwrap();
    let depth = case_index % GENERATED_DEPTH_MAX + 1;
    let mut current = root.clone();
    let mut entry_count: u32 = 1;
    for level in 0..depth {
        let directory = current.join(format!("d-{level:02}"));
        std::fs::create_dir(&directory).unwrap();
        current = directory;
        entry_count = entry_count.saturating_add(1);
    }
    let file_bytes = case_index.saturating_add(1).saturating_mul(31).min(GENERATED_FILE_BYTES_MAX);
    let bytes: Vec<u8> = (0..file_bytes)
        .map(|offset| DETERMINISTIC_SEED.wrapping_add(case_index).wrapping_add(offset).to_le_bytes()[0])
        .collect();
    std::fs::write(current.join("payload"), &bytes).unwrap();
    entry_count = entry_count.saturating_add(1);
    assert!(entry_count <= GENERATED_ENTRY_COUNT_MAX);
    assert!(u32::try_from(bytes.len()).unwrap() <= GENERATED_FILE_BYTES_MAX);
    Fixture {
        _temporary: temporary,
        root,
    }
}

// r[verify store_transports.nix_archive_parity]
#[test]
fn generated_tree_parity_is_deterministic_and_bounded() {
    for case_index in 0..GENERATED_CASE_COUNT {
        let fixture = generated_tree(case_index);
        let first = adapter_bytes(&fixture.root);
        let second = adapter_bytes(&fixture.root);
        let snix = snix_nar(&fixture.root);
        assert_eq!(first, second, "generated case {case_index} was not deterministic");
        assert_eq!(first, snix, "generated case {case_index} differed from Snix");
    }
}

// r[verify store_transports.nix_archive_parity]
#[test]
fn generated_case_planning_rejects_the_first_case_over_the_bound() {
    let rejected_case = GENERATED_CASE_COUNT;
    let result = std::panic::catch_unwind(|| generated_tree(rejected_case));
    assert!(result.is_err());
    assert_eq!(rejected_case, GENERATED_CASE_COUNT);
}

// r[verify store_transports.nix_archive_parity]
#[test]
fn explicit_case_hack_collision_and_read_failure_are_typed_errors() {
    let temporary = tempfile::tempdir().unwrap();
    let collision = temporary.path().join("collision");
    std::fs::create_dir(&collision).unwrap();
    std::fs::write(collision.join("name"), b"first").unwrap();
    std::fs::write(collision.join("name~nix~case~hack~1"), b"second").unwrap();
    let collision_result = observe_path(
        &collision,
        FilesystemNarRequest::new(HashAlgo::Sha256, CaseHackPolicy::Enabled, FIXTURE_NAR_BYTES_MAX),
    );
    assert!(matches!(collision_result, Err(FilesystemNarError::Upstream(_))));

    let unreadable = temporary.path().join("unreadable");
    std::fs::write(&unreadable, b"private").unwrap();
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(NO_PERMISSIONS_MODE)).unwrap();
    let read_result = observe_path(
        &unreadable,
        FilesystemNarRequest::new(HashAlgo::Sha256, CaseHackPolicy::Disabled, FIXTURE_NAR_BYTES_MAX),
    );
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(OWNER_READ_WRITE_MODE)).unwrap();
    assert!(read_result.is_err());
    assert!(unreadable.is_file());
}

fn nix_dump(path: &Path) -> Result<Vec<u8>, String> {
    let version = Command::new("nix-store").arg("--version").output().map_err(|error| error.to_string())?;
    if !version.status.success() {
        return Err(String::from_utf8_lossy(&version.stderr).into_owned());
    }
    let output = Command::new("nix-store").arg("--dump").arg(path).output().map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(output.stdout)
}

// r[verify store_transports.nix_archive_parity]
#[test]
fn nix_oracle_is_reported_as_matched_or_unavailable() {
    let fixture = parity_fixture();
    let adapter = adapter_bytes(&fixture.root);
    let oracle = match nix_dump(&fixture.root) {
        Ok(bytes) => {
            assert_eq!(adapter, bytes);
            println!("nix_oracle=matched");
            OptionalOracleDisposition::Matched
        }
        Err(error) => {
            println!("nix_oracle=unavailable reason={error}");
            OptionalOracleDisposition::Unavailable
        }
    };
    assert!(matches!(oracle, OptionalOracleDisposition::Matched | OptionalOracleDisposition::Unavailable));
    assert!(!adapter.is_empty());
}

// r[verify store_transports.nix_archive_parity]
#[test]
fn actual_fixture_evidence_satisfies_the_cutover_gate() {
    let fixture = parity_fixture();
    let adapter = adapter_bytes(&fixture.root);
    let snix = snix_nar(&fixture.root);
    let case_id = "byte-safe-tree".to_string();
    let comparison = ParityComparison {
        case_id: case_id.clone(),
        bytes_match: adapter == snix,
        size_match: adapter.len() == snix.len(),
        digests_match: all_algorithms()
            .into_iter()
            .all(|algorithm| digest_bytes(&adapter, algorithm) == digest_bytes(&snix, algorithm)),
        errors_match: true,
    };
    let decision = evaluate_cutover(&CutoverEvidence {
        upstream_version: UPSTREAM_VERSION.to_string(),
        upstream_commit: UPSTREAM_COMMIT.to_string(),
        upstream_cargo_checksum: UPSTREAM_CARGO_CHECKSUM.to_string(),
        platform_supported: cfg!(unix),
        required_cases: vec![case_id],
        comparisons: vec![comparison],
        nix_oracle: OptionalOracleDisposition::Unavailable,
    });
    assert!(decision.accepted, "cutover rejections: {:?}", decision.rejections);
    assert!(decision.rejections.is_empty());
}
