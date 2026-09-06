// r[verify native_package_parity.gateway]
use crunch_build::distributed::GatewayCapability;
use nix_compat::wire::de::NixRead as _;
use sha2::Digest as _;

use super::*;

const TARGET: &str = "gateway-target";

struct Fixture {
    root: tempfile::TempDir,
    gateway: MantleNixGateway,
    request: AddToStoreNarRequest,
    nar: Vec<u8>,
}

fn config(root: &Path) -> crunch_store::StoreConfig {
    crunch_store::StoreConfig::new(root.join("state"), root.join("outputs"), "/nix/store".to_string())
}

async fn fixture() -> Fixture {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("outputs")).unwrap();
    let raw_key = ed25519_dalek::SigningKey::from_bytes(&[0x35; 32]);
    let key = VerifyingKey::new("gateway-test".into(), raw_key.verifying_key());
    let signing_key = nix_compat::narinfo::SigningKey::new("gateway-test".into(), raw_key);
    let store = crunch_store::GatewayStore::open(config(root.path())).await.unwrap();
    let actor = RemoteGatewayAuthority {
        source: crunch_build::distributed::GatewayAuthoritySource::VerifiedUcan,
        subject: "gateway-test".into(),
        account_scope: "project-a".into(),
        audience: "mantle-remote-gateway".into(),
        expires_unix_s: gateway_now_unix_s().unwrap().saturating_add(600),
        capabilities: vec![GatewayCapability::UploadInput, GatewayCapability::ReadStore],
        evidence_refs_blake3: vec![blake3::hash(b"fixture-authority").to_hex().to_string()],
    };
    let gateway = MantleNixGateway::new(store, actor, GatewayPolicy::default(), vec![key]).unwrap();
    let mut nar = Vec::new();
    nix_compat::nar::writer::open(&mut nar).unwrap().symlink(TARGET.as_bytes()).unwrap();
    assert!(!nar.is_empty());
    assert!(nar.len() < 4096);
    let hash: [u8; 32] = sha2::Sha256::digest(&nar).into();
    let mut hash_wire = 64_u64.to_le_bytes().to_vec();
    hash_wire.extend_from_slice(data_encoding::HEXLOWER.encode(&hash).as_bytes());
    let nar_hash = nix_compat::wire::de::NixReader::new(hash_wire.as_slice()).read_value().await.unwrap();
    let mut request = AddToStoreNarRequest {
        path: StorePath::from_bytes(b"11111111111111111111111111111111-gateway").unwrap(),
        deriver: None,
        nar_hash,
        references: Vec::new(),
        registration_time: 0,
        nar_size: u64::try_from(nar.len()).unwrap(),
        ultimate: false,
        signatures: Vec::new(),
        ca: None,
        repair: false,
        dont_check_sigs: false,
    };
    let mut info = path_info_from_import(&request, node(), hash);
    crunch_build::signing::sign_pathinfo(&mut info, &signing_key);
    request.signatures = info.signatures;
    Fixture {
        root,
        gateway,
        request,
        nar,
    }
}

fn node() -> snix_castore::Node {
    snix_castore::Node::Symlink {
        target: TARGET.try_into().unwrap(),
    }
}

async fn assert_not_published(fixture: &Fixture, path: &StorePath<String>) {
    assert!(fixture.gateway.store.lock().await.find(path).await.unwrap().is_none());
    assert_eq!(fs::read_dir(fixture.root.path().join("outputs")).unwrap().count(), 0);
}

#[tokio::test]
async fn admitted_import_survives_reopen_and_does_not_register_a_root() {
    let fixture = fixture().await;
    let path = fixture.request.path.clone();
    let size = fixture.request.nar_size;
    let hash = *fixture.request.nar_hash;
    fixture.gateway.add_to_store_nar(fixture.request, &mut fixture.nar.as_slice()).await.unwrap();
    let found = fixture.gateway.query_path_info(&path).await.unwrap().unwrap();
    assert_eq!(found.nar_size, size);
    assert!(fixture.gateway.query_path_from_hash_part(path.digest()).await.unwrap().is_some());
    // Imported objects retain the existing castore-only, non-root behavior.
    assert_eq!(fs::read_dir(fixture.root.path().join("outputs")).unwrap().count(), 0);
    assert!(!fixture.root.path().join("state/gc-roots.json").exists());
    drop(fixture.gateway);
    let reopened = crunch_store::GatewayStore::open(config(fixture.root.path())).await.unwrap();
    let info = reopened.find_by_digest(*path.digest()).await.unwrap().unwrap();
    assert_eq!(info.store_path, path);
    assert_eq!(info.nar_sha256, hash);
    assert_eq!(info.node, node());
}

#[tokio::test]
async fn trust_overrides_and_denied_authority_do_not_read_the_nar() {
    for case in 0..3 {
        let mut fixture = fixture().await;
        let path = fixture.request.path.clone();
        match case {
            0 => fixture.request.dont_check_sigs = true,
            1 => fixture.request.repair = true,
            _ => fixture.gateway.authority.capabilities = vec![GatewayCapability::ReadStore],
        }
        let mut reader = std::io::Cursor::new(&fixture.nar);
        let result = fixture.gateway.add_to_store_nar(fixture.request, &mut reader).await;
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(reader.position(), 0);
        assert!(fixture.gateway.store.lock().await.find(&path).await.unwrap().is_none());
        assert_eq!(fs::read_dir(fixture.root.path().join("outputs")).unwrap().count(), 0);
    }
}

#[tokio::test]
async fn changed_nar_size_hash_and_signature_never_publish_an_output() {
    for case in 0..4 {
        let mut fixture = fixture().await;
        let path = fixture.request.path.clone();
        match case {
            0 => fixture.request.nar_size = fixture.request.nar_size.saturating_add(1),
            1 => {
                let index = fixture.nar.windows(TARGET.len()).position(|bytes| bytes == TARGET.as_bytes()).unwrap();
                fixture.nar[index] = b'h';
            }
            2 => {
                fixture.gateway.trusted_store_keys = vec![VerifyingKey::new(
                    "wrong-key".into(),
                    ed25519_dalek::SigningKey::from_bytes(&[0x36; 32]).verifying_key(),
                )]
            }
            _ => fixture.request.nar_size = fixture.request.nar_size.saturating_sub(1),
        }
        let error = fixture.gateway.add_to_store_nar(fixture.request, &mut fixture.nar.as_slice()).await.unwrap_err();
        if case < 2 {
            assert!(error.to_string().contains("nix-gateway-nar-identity-mismatch"), "{error}");
        } else if case == 2 {
            assert!(error.to_string().contains("nix-gateway-pathinfo-signature-untrusted"), "{error}");
        } else {
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        }
        assert!(fixture.gateway.store.lock().await.find(&path).await.unwrap().is_none());
        assert_eq!(fs::read_dir(fixture.root.path().join("outputs")).unwrap().count(), 0);
    }
}

#[tokio::test]
async fn final_persistence_rechecks_authority_after_receiving_the_nar() {
    let mut fixture = fixture().await;
    let path = fixture.request.path.clone();
    let received = fixture
        .gateway
        .store
        .lock()
        .await
        .ingest_nar(&mut fixture.nar.as_slice(), &None, fixture.request.nar_size)
        .await
        .unwrap();
    let info = path_info_from_import(&fixture.request, node(), *fixture.request.nar_hash);
    let operation = GatewayOperation::UploadStoreObject {
        logical_path: path.to_absolute_path(),
        nar_size_bytes: info.nar_size,
        nar_sha256_hex: data_encoding::HEXLOWER.encode(&info.nar_sha256),
        signature_count: 1,
    };
    fixture.gateway.authorize(operation.clone()).unwrap();
    fixture.gateway.authority.expires_unix_s = 1;
    let error = fixture
        .gateway
        .persist_verified_import(operation, crunch_store::GatewayImportRequest {
            path_info: info,
            received,
        })
        .await
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    assert_not_published(&fixture, &path).await;
}

#[tokio::test]
async fn ingest_observation_cannot_cross_store_instances() {
    let source = fixture().await;
    let destination = fixture().await;
    let path = source.request.path.clone();
    let received = source
        .gateway
        .store
        .lock()
        .await
        .ingest_nar(&mut source.nar.as_slice(), &None, source.request.nar_size)
        .await
        .unwrap();
    let path_info = path_info_from_import(&source.request, node(), *source.request.nar_hash);
    let result = destination
        .gateway
        .store
        .lock()
        .await
        .persist_import(crunch_store::GatewayImportRequest { path_info, received })
        .await;
    assert!(result.is_err(), "an observation from another store cannot authorize local metadata");
    assert_not_published(&destination, &path).await;
}

#[tokio::test]
async fn zero_ingest_limit_rejects_before_reading() {
    let fixture = fixture().await;
    let mut reader = std::io::Cursor::new(&fixture.nar);
    let result = fixture.gateway.store.lock().await.ingest_nar(&mut reader, &None, 0).await;
    assert!(result.is_err());
    assert_eq!(reader.position(), 0);
    assert_not_published(&fixture, &fixture.request.path).await;
}

#[tokio::test]
async fn store_capability_rejects_metadata_not_bound_to_its_observed_nar() {
    for case in 0..3 {
        let fixture = fixture().await;
        let path = fixture.request.path.clone();
        let received = fixture
            .gateway
            .store
            .lock()
            .await
            .ingest_nar(&mut fixture.nar.as_slice(), &None, fixture.request.nar_size)
            .await
            .unwrap();
        let mut info = path_info_from_import(&fixture.request, node(), *fixture.request.nar_hash);
        match case {
            0 => {
                info.node = snix_castore::Node::Symlink {
                    target: "other".try_into().unwrap(),
                }
            }
            1 => info.nar_size = info.nar_size.saturating_add(1),
            _ => info.nar_sha256 = [0; 32],
        }
        let error = fixture
            .gateway
            .store
            .lock()
            .await
            .persist_import(crunch_store::GatewayImportRequest {
                path_info: info,
                received,
            })
            .await
            .unwrap_err();
        assert!(error.to_string().contains("differs from observed NAR"));
        assert_not_published(&fixture, &path).await;
    }
}
