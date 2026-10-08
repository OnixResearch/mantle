#![cfg(target_os = "linux")]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io;
use std::num::NonZeroUsize;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crunch_build::{HermeticityMode, KeyPair, action_result, signing};
use crunch_glue::{ConversionCache, CrunchDerivation, Input};
use crunch_nix_gateway::{handshake, read_request, store::VerifiedStore, write_rejection};
use crunch_nix_gateway_core::Reject;
use crunch_store::{StoreHandle, StoreHandleServices};
use nix_compat::derivation::{Derivation, Output};
use nix_compat::narinfo::{SigningKey, VerifyingKey, fingerprint_with_store_dir};
use nix_compat::nixhash::{CAHash, NixHash};
use nix_compat::store_path::{StorePath, StorePathRef};
use snix_castore::blobservice::{BlobService, MemoryBlobService};
use snix_castore::directoryservice::{DirectoryService, RedbDirectoryService, RedbDirectoryServiceConfig};
use snix_castore::{Node, SymlinkTarget};
use snix_store::nar::{NarCalculationService, SimpleRenderer};
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::{LruPathInfoService, PathInfoService};
use tokio::io::AsyncWriteExt;

#[derive(Clone, Copy)]
enum Case { Signed, Missing, Tampered, SignedWrongNar, WrongCa, IncompleteCas }

fn fixture(case: Case, temp: &tempfile::TempDir, runtime: &tokio::runtime::Runtime) -> (VerifiedStore, String) {
    let blobs = Arc::new(MemoryBlobService::default()) as Arc<dyn BlobService>;
    let directories = Arc::new(RedbDirectoryService::new_temporary(
        "gateway-nix-test".to_string(), RedbDirectoryServiceConfig::default(),
    ).unwrap()) as Arc<dyn DirectoryService>;
    let infos = Arc::new(LruPathInfoService::with_capacity(
        "gateway-nix-test".to_string(), NonZeroUsize::new(32).unwrap(),
    )) as Arc<dyn PathInfoService>;
    let path = StorePath::from_name_and_digest_fixed("gateway-client", [7_u8; 20]).unwrap();
    let absolute_path = path.to_absolute_path();
    let raw_key = ed25519_dalek::SigningKey::from_bytes(&[13_u8; 32]);
    let signer = SigningKey::new("gateway-fixture-1".to_string(), raw_key.clone());
    let trusted = VerifyingKey::new("gateway-fixture-1".to_string(), raw_key.verifying_key());
    if !matches!(case, Case::Missing) {
        let node = Node::Symlink { target: SymlinkTarget::try_from("actual-content").unwrap() };
        let (nar_size, nar_sha256) = runtime.block_on(
            SimpleRenderer::new(blobs.clone(), directories.clone()).calculate_nar(&node),
        ).unwrap();
        let mut info = PathInfo {
            store_path: path, node, references: Vec::new(), nar_size, nar_sha256,
            signatures: Vec::new(), deriver: None, ca: None,
        };
        if matches!(case, Case::WrongCa) {
            info.ca = Some(CAHash::Nar(NixHash::Sha256(nar_sha256)));
        }
        if matches!(case, Case::IncompleteCas) {
            info.node = Node::File {
                digest: (&[0xa5_u8; 32][..]).try_into().unwrap(),
                size: 1,
                executable: false,
            };
        }
        if matches!(case, Case::SignedWrongNar) {
            // A real configured signer cannot bless content missing its claimed NAR.
            info.nar_size += 1;
        }
        let refs: Vec<StorePathRef<'_>> = Vec::new();
        let fingerprint = fingerprint_with_store_dir(
            &info.store_path.as_ref(), &info.nar_sha256, info.nar_size, refs.iter(), "/nix/store",
        );
        info.signatures.push(signer.sign(fingerprint.as_bytes()).to_owned());
        if matches!(case, Case::Tampered) { info.nar_size += 1; }
        runtime.block_on(infos.put(info)).unwrap();
    }
    let handle = StoreHandle::from_services_with_store_dir(StoreHandleServices {
        blob_service: blobs, directory_service: directories, pathinfo_service: infos,
        remote_pathinfo: None, state_dir: temp.path().to_path_buf(),
        output_dir_str: temp.path().display().to_string(), publishers: Vec::new(),
    }, "/nix/store".to_string());
    (VerifiedStore::new(handle, vec![trusted]).unwrap(), absolute_path)
}

#[test]
fn signed_path_info_is_bound_to_measured_cas_content() {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    for (case, expected) in [
        (Case::Signed, Ok(())),
        (Case::SignedWrongNar, Err(Reject::Authority)),
        (Case::IncompleteCas, Err(Reject::Authority)),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let (store, path) = fixture(case, &temp, &runtime);
        let observed = runtime.block_on(store.query_missing(&[path])).map(|missing| {
            assert!(missing.is_empty());
        });
        assert_eq!(observed, expected);
    }
}


#[derive(Clone, Copy)]
enum CachedCase {
    Signed,
    MissingRecord,
    ForgedRecordSignature,
    AlteredDrvBytes,
    AlteredOutput,
    AmbiguousRecords,
}

async fn cached_drv_fixture(case: CachedCase, temp: &tempfile::TempDir) -> (VerifiedStore, String, String) {
    let blobs = Arc::new(MemoryBlobService::default()) as Arc<dyn BlobService>;
    let directories = Arc::new(RedbDirectoryService::new_temporary(
        "gateway-cached-derivation".to_string(), RedbDirectoryServiceConfig::default(),
    ).unwrap()) as Arc<dyn DirectoryService>;
    let infos = Arc::new(LruPathInfoService::with_capacity(
        "gateway-cached-derivation".to_string(), NonZeroUsize::new(16).unwrap(),
    )) as Arc<dyn PathInfoService>;
    let renderer = SimpleRenderer::new(blobs.clone(), directories.clone());
    let raw_key = ed25519_dalek::SigningKey::from_bytes(&[13_u8; 32]);
    let keypair = KeyPair {
        signing_key: SigningKey::new("gateway-fixture-1".to_string(), raw_key.clone()),
        verifying_key: VerifyingKey::new("gateway-fixture-1".to_string(), raw_key.verifying_key()),
    };
    let output_path = StorePath::from_name_and_digest_fixed("cached-output", [7_u8; 20]).unwrap();
    let derivation = Derivation {
        arguments: vec!["-c".to_string(), "echo cached > \"$out\"".to_string()],
        builder: "/bin/sh".to_string(),
        environment: BTreeMap::from([("name".to_string(), b"cached-output".to_vec().into())]),
        input_derivations: BTreeMap::new(),
        input_sources: BTreeSet::new(),
        outputs: BTreeMap::from([("out".to_string(), Output {
            path: Some(output_path.clone()), ca_hash: None,
        })]),
        system: "x86_64-linux".to_string(),
    };
    let at = derivation.to_aterm_bytes_with_store_dir("/nix/store");
    let mut writer = blobs.open_write().await;
    writer.write_all(&at).await.unwrap();
    let digest = writer.close().await.unwrap();
    let drv_node = Node::File { digest, size: at.len() as u64, executable: false };
    let (nar_size, nar_sha256) = renderer.calculate_nar(&drv_node).await.unwrap();
    let drv_path = StorePath::from_name_and_digest_fixed("cached-output.drv", [11_u8; 20]).unwrap();
    let mut drv_info = PathInfo {
        store_path: drv_path.clone(), node: drv_node,
        references: Vec::new(), nar_size, nar_sha256,
        signatures: Vec::new(), deriver: None, ca: None,
    };
    signing::sign_pathinfo_with_store_dir(&mut drv_info, &keypair.signing_key, "/nix/store");
    if matches!(case, CachedCase::AlteredDrvBytes) {
        let mut writer = blobs.open_write().await;
        let altered = b"not-the-signed-aterm";
        writer.write_all(altered).await.unwrap();
        let tampered = writer.close().await.unwrap();
        drv_info.node = Node::File { digest: tampered, size: altered.len() as u64, executable: false };
    }
    infos.put(drv_info).await.unwrap();
    let output_node = Node::Symlink {
        target: SymlinkTarget::try_from("genuine-content").unwrap(),
    };
    let (size, hash) = renderer.calculate_nar(&output_node).await.unwrap();
    let mut output = PathInfo {
        store_path: output_path.clone(), node: output_node,
        references: Vec::new(), nar_size: size, nar_sha256: hash,
        signatures: Vec::new(), deriver: None, ca: None,
    };
    signing::sign_pathinfo_with_store_dir(&mut output, &keypair.signing_key, "/nix/store");
    let second_signer = matches!(case, CachedCase::AmbiguousRecords).then(|| {
        let secret = ed25519_dalek::SigningKey::from_bytes(&[23_u8; 32]);
        KeyPair {
            signing_key: SigningKey::new("gateway-fixture-2".to_string(), secret.clone()),
            verifying_key: VerifyingKey::new("gateway-fixture-2".to_string(), secret.verifying_key()),
        }
    });
    if let Some(other) = &second_signer {
        signing::sign_pathinfo_with_store_dir(&mut output, &other.signing_key, "/nix/store");
    }
    let original_outputs = BTreeMap::from([("out".to_string(), output.clone())]);
    let signed = action_result::signed_record_for_outputs(
        &derivation, &original_outputs, "/nix/store", HermeticityMode::Strict, &keypair,
    ).unwrap();
    if matches!(case, CachedCase::AlteredOutput) {
        output.nar_size += 1;
    }
    infos.put(output).await.unwrap();
    let handle = StoreHandle::from_services_with_store_dir(StoreHandleServices {
        blob_service: blobs, directory_service: directories, pathinfo_service: infos.clone(),
        remote_pathinfo: None, state_dir: temp.path().to_path_buf(),
        output_dir_str: temp.path().display().to_string(), publishers: Vec::new(),
    }, "/nix/store".to_string());
    if !matches!(case, CachedCase::MissingRecord) {
        let mut signed = signed;
        if matches!(case, CachedCase::ForgedRecordSignature) {
            let imposter = SigningKey::new(
                keypair.verifying_key.name().to_string(),
                ed25519_dalek::SigningKey::from_bytes(&[23_u8; 32]),
            );
            signed.record_signatures[0].signature = imposter.sign(signed.record.result_ref.as_bytes()).to_string();
        }
        handle.publish_local_action_result(&signed).await.unwrap();
    }
    if let Some(other) = &second_signer {
        let second = action_result::signed_record_for_outputs(
            &derivation, &original_outputs, "/nix/store", HermeticityMode::Strict, other,
        ).unwrap();
        handle.publish_local_action_result(&second).await.unwrap();
    }
    let drv = drv_path.to_absolute_path();
    let output = output_path.to_absolute_path();
    let mut trusted = vec![keypair.verifying_key];
    if let Some(other) = second_signer {
        trusted.push(other.verifying_key);
    }
    (VerifiedStore::new(handle, trusted).unwrap(), drv, output)
}

#[test]
fn cached_derived_query_requires_signed_drv_and_unambiguous_complete_output() {
    for (case, expected) in [
        (CachedCase::Signed, Ok(())),
        (CachedCase::MissingRecord, Err(Reject::Operation)),
        (CachedCase::ForgedRecordSignature, Err(Reject::Authority)),
        (CachedCase::AlteredDrvBytes, Err(Reject::Authority)),
        (CachedCase::AlteredOutput, Err(Reject::Authority)),
        (CachedCase::AmbiguousRecords, Err(Reject::Authority)),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let (store, drv, output) = runtime.block_on(cached_drv_fixture(case, &temp));
        let target = format!("{drv}!*");
        let actual = runtime.block_on(store.query_missing(&[target])).map(|paths| {
            assert!(paths.is_empty(), "signed cached output should not appear missing");
        });
        assert_eq!(actual, expected);
        if matches!(case, CachedCase::Signed) {
            assert!(runtime.block_on(store.query_missing(&[output])).unwrap().is_empty());
            assert_eq!(
                runtime.block_on(store.query_missing(&[format!("{drv}!out")])).unwrap(),
                Vec::<String>::new(),
            );
            assert_eq!(
                runtime.block_on(store.query_missing(&[format!("{drv}!dev")])),
                Err(Reject::Operation),
            );
            let mut wire = Vec::new();
            assert_eq!(
                runtime.block_on(store.answer(
                    crunch_nix_gateway::ParsedRequest::BuildPaths { paths: vec![format!("{drv}!*")] },
                    &mut wire,
                )),
                Err(Reject::Operation),
            );
            assert!(wire.is_empty(), "cached output bypassed Build authorization");
        }
    }
}

#[derive(Clone, Copy)]
enum ProjectionCase { SignedIa, MissingSource, ChangedEnv, ChangedOutput, DifferentDrvPath }

async fn generated_signed_ia_fixture(
    case: ProjectionCase,
    temp: &tempfile::TempDir,
) -> (VerifiedStore, StorePath<String>, StorePath<String>) {
    let blobs = Arc::new(MemoryBlobService::default()) as Arc<dyn BlobService>;
    let directories = Arc::new(RedbDirectoryService::new_temporary(
        "gateway-remote-projection".to_string(), RedbDirectoryServiceConfig::default(),
    ).unwrap()) as Arc<dyn DirectoryService>;
    let infos = Arc::new(LruPathInfoService::with_capacity(
        "gateway-remote-projection".to_string(), NonZeroUsize::new(16).unwrap(),
    )) as Arc<dyn PathInfoService>;
    let renderer = SimpleRenderer::new(blobs.clone(), directories.clone());
    let raw_key = ed25519_dalek::SigningKey::from_bytes(&[13_u8; 32]);
    let signer = SigningKey::new("gateway-fixture-1".to_string(), raw_key.clone());
    let verifier = VerifyingKey::new("gateway-fixture-1".to_string(), raw_key.verifying_key());
    let source = StorePath::from_name_and_digest_fixed("declared-input", [0x48_u8; 20]).unwrap();
    let crunch = CrunchDerivation {
        name: "signed-input-addressed".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec!["-c".to_string(), "printf result > \"$out\"".to_string()],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: Vec::new(),
        env: HashMap::new(),
        inputs: vec![Input::Source(source.to_absolute_path())],
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };
    let (original_path, mut nix) = crunch_glue::convert(&crunch, &mut ConversionCache::new("/nix/store")).unwrap();
    if matches!(case, ProjectionCase::ChangedEnv) {
        nix.environment.insert("attack".to_string(), b"changed".to_vec().into());
    }
    if matches!(case, ProjectionCase::ChangedOutput) {
        nix.outputs.get_mut("out").unwrap().path =
            Some(StorePath::from_name_and_digest_fixed("other-output", [0x17_u8; 20]).unwrap());
    }
    let drv_path = if matches!(case, ProjectionCase::DifferentDrvPath) {
        StorePath::from_name_and_digest_fixed("unrelated.drv", [0x17_u8; 20]).unwrap()
    } else {
        original_path
    };
    let at = nix.to_aterm_bytes_with_store_dir("/nix/store");
    let mut writer = blobs.open_write().await;
    writer.write_all(&at).await.unwrap();
    let digest = writer.close().await.unwrap();
    let node = Node::File { digest, size: at.len() as u64, executable: false };
    let (nar_size, nar_sha256) = renderer.calculate_nar(&node).await.unwrap();
    let mut drv_info = PathInfo {
        store_path: drv_path.clone(), node, nar_size, nar_sha256,
        references: Vec::new(), signatures: Vec::new(), deriver: None, ca: None,
    };
    signing::sign_pathinfo_with_store_dir(&mut drv_info, &signer, "/nix/store");
    infos.put(drv_info).await.unwrap();
    if !matches!(case, ProjectionCase::MissingSource) {
        let node = Node::Symlink {
            target: SymlinkTarget::try_from("measured-source-content").unwrap(),
        };
        let (nar_size, nar_sha256) = renderer.calculate_nar(&node).await.unwrap();
        let mut info = PathInfo {
            store_path: source.clone(), node, nar_size, nar_sha256,
            references: Vec::new(), signatures: Vec::new(), deriver: None, ca: None,
        };
        signing::sign_pathinfo_with_store_dir(&mut info, &signer, "/nix/store");
        infos.put(info).await.unwrap();
    }
    let handle = StoreHandle::from_services_with_store_dir(StoreHandleServices {
        blob_service: blobs, directory_service: directories, pathinfo_service: infos,
        remote_pathinfo: None, state_dir: temp.path().to_path_buf(),
        output_dir_str: temp.path().display().to_string(), publishers: Vec::new(),
    }, "/nix/store".to_string());
    (VerifiedStore::new(handle, vec![verifier]).unwrap(), drv_path, source)
}

#[test]
fn signed_ia_projection_requires_exact_remote_derivation_and_measured_inputs() {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    for (case, expected) in [
        (ProjectionCase::SignedIa, Ok(())),
        (ProjectionCase::MissingSource, Err(Reject::Operation)),
        (ProjectionCase::ChangedEnv, Err(Reject::Identity)),
        (ProjectionCase::ChangedOutput, Err(Reject::Identity)),
        (ProjectionCase::DifferentDrvPath, Err(Reject::Identity)),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let (store, drv_path, source) = runtime.block_on(generated_signed_ia_fixture(case, &temp));
        let actual = runtime.block_on(store.project_signed_ia_for_remote(&drv_path.to_absolute_path()))
            .map(|proof| {
                let (path, crunch, nix) = proof.into_derivations();
                assert_eq!(path, drv_path);
                assert_eq!(crunch.name, "signed-input-addressed");
                assert!(nix.input_sources.contains(&source));
                assert!(nix.outputs["out"].path.is_some());
            });
        assert_eq!(actual, expected);
    }
    let temp = tempfile::tempdir().unwrap();
    let (store, drv, _) = runtime.block_on(cached_drv_fixture(CachedCase::Signed, &temp));
    let error = runtime.block_on(store.project_signed_ia_for_remote(&drv)).err();
    assert_eq!(error, Some(Reject::Identity));
}

#[test]
fn build_target_projection_requires_signed_ia_and_exact_selected_output_set() {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let (store, drv, source) = runtime.block_on(generated_signed_ia_fixture(ProjectionCase::SignedIa, &temp));
    let drv = drv.to_absolute_path();
    let target = format!("{drv}!*");
    let proven = runtime.block_on(store.project_build_paths_for_remote(&[target.clone()])).unwrap();
    let (projected_drv, crunch, nix) = proven.into_iter().next().unwrap().into_derivations();
    assert_eq!(projected_drv.to_absolute_path(), drv);
    assert!(nix.input_sources.contains(&source));
    assert_eq!(crunch.outputs, vec!["out"]);
    for (targets, rejection) in [
        (vec![format!("{drv}!dev")], Reject::Operation),
        (vec![format!("{drv}!out,out")], Reject::Operation),
        (vec![target.clone(), format!("{drv}!out")], Reject::Conflict),
        (vec![drv.clone()], Reject::Identity),
        (Vec::new(), Reject::Bound),
    ] {
        assert_eq!(runtime.block_on(store.project_build_paths_for_remote(&targets)).err(), Some(rejection));
    }
    let missing = tempfile::tempdir().unwrap();
    let (unproven, drv, _) = runtime.block_on(generated_signed_ia_fixture(ProjectionCase::MissingSource, &missing));
    assert_eq!(
        runtime
            .block_on(unproven.project_build_paths_for_remote(&[format!("{}!*", drv.to_absolute_path())]))
            .err(),
        Some(Reject::Operation),
    );
}

/// The Nix client may ask whether a derivation is cached, but without a
/// verified Build credential it must not receive an opcode-9 success reply.
#[test]
#[ignore = "requires an installed Nix 2.36 client and private Unix sockets"]
fn installed_nix_cached_drv_never_acks_unauthorized_build_paths() {
    let temp = tempfile::tempdir().unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let (store, drv, _) = runtime.block_on(cached_drv_fixture(CachedCase::Signed, &temp));
    let private = temp.path().join("private");
    std::fs::create_dir(&private).unwrap();
    std::fs::set_permissions(&private, std::fs::Permissions::from_mode(0o700)).unwrap();
    let socket = private.join("build.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
    let server = std::thread::spawn(move || -> io::Result<Vec<String>> {
        let (mut stream, _) = listener.accept()?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;
        handshake(&mut stream)?;
        let mut observed = Vec::new();
        for _ in 0..8 {
            let request = match read_request(&mut stream) {
                Ok(request) => request,
                Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(error) => return Err(error),
            };
            observed.push(format!("{request:?}"));
            if let Err(reason) = runtime.block_on(store.answer(request, &mut stream)) {
                write_rejection(&mut stream, reason)?;
                break;
            }
        }
        Ok(observed)
    });
    let mut client = Command::new("nix-store").args(["--realise", "--store"])
        .arg(format!("unix://{}", socket.display())).arg(&drv)
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while client.try_wait().unwrap().is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    if client.try_wait().unwrap().is_none() { client.kill().unwrap(); }
    let output = client.wait_with_output().unwrap();
    let observed = server.join().unwrap().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(observed.iter().any(|request| request.starts_with("QueryMissing")),
        "native Nix did not consume a cached derivation query: {observed:?}; {stderr}");
    assert!(observed.iter().any(|request| request.starts_with("BuildPaths")),
        "native Nix skipped the Build permission gate: {observed:?}; {stderr}");
    assert!(!output.status.success(), "unauthorized build succeeded: {observed:?}");
    assert!(stderr.contains(Reject::Operation.code()), "{stderr}; {observed:?}");
}

#[test]
fn private_listener_rejects_shared_socket_directory() {
    use std::sync::atomic::AtomicBool;

    let temp = tempfile::tempdir().unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let (store, _) = fixture(Case::Signed, &temp, &runtime);
    let shared = temp.path().join("shared");
    std::fs::create_dir(&shared).unwrap();
    std::fs::set_permissions(&shared, std::fs::Permissions::from_mode(0o755)).unwrap();
    let socket = shared.join("worker.sock");
    let error = crunch_nix_gateway::server::serve_private(&socket, store, &AtomicBool::new(false))
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    assert!(!socket.exists(), "unauthorized listener left a socket behind");
}

/// An actual installed Nix client consumes both response messages; merely
/// decoding its first request would not establish store interoperability.
#[test]
#[ignore = "requires an installed Nix 2.36 client and private Unix sockets"]
fn installed_nix_path_info_requires_signed_complete_store() {
    for (case, label) in [
        (Case::Signed, "signed"), (Case::Missing, "missing"),
        (Case::Tampered, "tampered"), (Case::SignedWrongNar, "signed-wrong-nar"),
        (Case::WrongCa, "wrong-ca"), (Case::IncompleteCas, "incomplete-cas"),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let (store, path) = fixture(case, &temp, &runtime);
        let socket = temp.path().join(format!("{label}.sock"));
        let listener = UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        let server = std::thread::spawn(move || -> io::Result<Vec<String>> {
            let (mut stream, _) = listener.accept()?;
            stream.set_read_timeout(Some(Duration::from_secs(5)))?;
            stream.set_write_timeout(Some(Duration::from_secs(5)))?;
            handshake(&mut stream)?;
            let mut operations = Vec::new();
            for _ in 0..8 {
                let request = match read_request(&mut stream) {
                    Ok(request) => request,
                    Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => break,
                    Err(error) => return Err(error),
                };
                operations.push(format!("{request:?}"));
                if let Err(reason) = runtime.block_on(store.answer(request, &mut stream)) {
                    write_rejection(&mut stream, reason)?;
                    break;
                }
            }
            Ok(operations)
        });
        let mut client = Command::new("nix").args(["path-info", "--store"])
            .arg(format!("unix://{}", socket.display())).arg(&path)
            .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while client.try_wait().unwrap().is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        if client.try_wait().unwrap().is_none() { client.kill().unwrap(); }
        let output = client.wait_with_output().unwrap();
        let observed = server.join().unwrap().unwrap();
        assert!(!observed.is_empty(), "{label}: Nix did not issue a store request");
        let stderr = String::from_utf8_lossy(&output.stderr);
        match case {
            Case::Signed => {
                assert!(output.status.success(), "{label}: {stderr}; requests: {observed:?}");
                assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), path);
                assert!(observed.iter().any(|request| request.starts_with("QueryPathInfo")),
                    "Nix never consumed signed PathInfo: {observed:?}");
            }
            Case::Missing => assert!(!output.status.success(), "missing PathInfo claimed present"),
            Case::Tampered | Case::SignedWrongNar | Case::WrongCa | Case::IncompleteCas => {
                assert!(!output.status.success(), "{label} store facts claimed present");
                assert!(stderr.contains(Reject::Authority.code()), "{stderr}; requests: {observed:?}");
            }
        }
    }
}

/// A malformed peer must not take down the owner-only store-read listener.
#[test]
#[ignore = "requires an installed Nix 2.36 client and private Unix sockets"]
fn private_listener_survives_malformed_peer_before_signed_read() {
    use std::io::Write as _;
    use std::os::unix::net::UnixStream;
    use std::sync::atomic::{AtomicBool, Ordering};

    let temp = tempfile::tempdir().unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let (store, path) = fixture(Case::Signed, &temp, &runtime);
    drop(runtime);
    let private_dir = temp.path().join("private-gateway");
    std::fs::create_dir(&private_dir).unwrap();
    std::fs::set_permissions(&private_dir, std::fs::Permissions::from_mode(0o700)).unwrap();
    let socket = private_dir.join("recover.sock");
    let stop = Arc::new(AtomicBool::new(false));
    let stop_server = Arc::clone(&stop);
    let server_socket = socket.clone();
    let server = std::thread::spawn(move || {
        crunch_nix_gateway::server::serve_private(&server_socket, store, &stop_server)
    });
    let deadline = Instant::now() + Duration::from_secs(5);
    while !socket.exists() && Instant::now() < deadline {
        if server.is_finished() {
            panic!("private listener exited before bind: {:?}", server.join().unwrap());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(socket.exists(), "private service did not bind before deadline");
    assert_eq!(std::fs::metadata(&socket).unwrap().permissions().mode() & 0o777, 0o600);
    let mut invalid = UnixStream::connect(&socket).unwrap();
    invalid.write_all(&[0_u8; 8]).unwrap();
    drop(invalid);
    let mut client = Command::new("nix").args(["path-info", "--store"])
        .arg(format!("unix://{}", socket.display())).arg(&path)
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while client.try_wait().unwrap().is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    if client.try_wait().unwrap().is_none() { client.kill().unwrap(); }
    let output = client.wait_with_output().unwrap();
    stop.store(true, Ordering::Release);
    server.join().unwrap().unwrap();
    assert!(output.status.success(), "client stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), path);
    assert!(!socket.exists(), "owner-only socket remained after clean shutdown");
}
