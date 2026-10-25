//! Real source admission scenarios: durable names never replace signed content checks.
use std::collections::BTreeMap;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;

use crunch_remote_core::endpoint::EndpointBinding;
use crunch_remote_core::endpoint::EndpointBindingSpec;
use crunch_remote_core::endpoint::EndpointKind;
use crunch_store::PullOptions;
use crunch_store::PushOptions;
use crunch_store::StoreBackend;
use crunch_store::StoreConfig;
use crunch_store::StoreHandle;
use crunch_store::StoreSource;
use crunch_store::StoreSourceAdmission;
use nix_compat::narinfo::SigningKey;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::narinfo::fingerprint_with_store_dir;
use nix_compat::store_path::StorePath;
use serde_json::Value;
use serde_json::json;
use sha2::Digest;
use sha2::Sha256;
use snix_store::path_info::PathInfo;

use super::super::endpoint::EndpointBindingStore;
use super::super::endpoint_config::REMOTE_ENDPOINT_BINDING_CONFIG_SCHEMA;
use super::super::endpoint_config::RemoteEndpointBindingConfig;

const PREFIX: &str = "/mantle/store";
const CONTENT: &[u8] = b"source binding does not replace signed NAR verification\n";

fn signer() -> SigningKey<ed25519_dalek::SigningKey> {
    SigningKey::new("source-binding-fixture-1".into(), ed25519_dalek::SigningKey::from_bytes(&[71; 32]))
}

fn verifying_key() -> VerifyingKey {
    let secret = ed25519_dalek::SigningKey::from_bytes(&[71; 32]);
    VerifyingKey::new("source-binding-fixture-1".into(), ed25519_dalek::VerifyingKey::from(&secret))
}

fn local_config(root: &Path) -> StoreConfig {
    StoreConfig::new(StoreBackend::Snix, root.join("state"), root.join("output"), PREFIX.into())
}

fn binding(store: &EndpointBindingStore, name: &str, kind: EndpointKind) -> EndpointBinding {
    store
        .create(
            EndpointBindingSpec {
                kind,
                name: name.into(),
                scope: "fixture-sources".into(),
                trust_requirement: "source-binding-fixture-1".into(),
                store_prefix: PREFIX.into(),
                expires_unix_s: u64::MAX,
            },
            None,
            crate::unix_time_now_s().unwrap(),
        )
        .unwrap()
}

fn row(source: &str, binding: &EndpointBinding) -> Value {
    json!({"source": source, "endpoint_binding": binding,
        "scope": "fixture-sources", "trust_requirement": "source-binding-fixture-1"})
}

fn load_config(root: &Path, state: &Path, rows: Vec<Value>) -> RemoteEndpointBindingConfig {
    let path = root.join("source-config.json");
    let mut file = std::fs::OpenOptions::new().create_new(true).write(true).mode(0o600).open(&path).unwrap();
    serde_json::to_writer(
        &mut file,
        &json!({
            "schema": REMOTE_ENDPOINT_BINDING_CONFIG_SCHEMA,
            "endpoint_state_directory": state, "source_bindings": rows,
        }),
    )
    .unwrap();
    drop(file);
    RemoteEndpointBindingConfig::load(&path).unwrap()
}

async fn signed_file(store: &mut StoreHandle, root: &Path, name: &str, digest: u8) -> PathInfo {
    let source = root.join(format!("{name}.txt"));
    std::fs::write(&source, CONTENT).unwrap();
    let path = StorePath::<String>::from_name_and_digest_fixed(name, [digest; 20]).unwrap();
    store
        .ingest_verified_source(crunch_store::VerifiedSourceIngestRequest {
            source_path: &source,
            logical_store_path: &path.to_absolute_path_with_prefix(PREFIX),
            source_name: name,
            signing_key: &signer(),
        })
        .await
        .unwrap()
}

async fn signed_cache(root: &Path) -> (PathInfo, BTreeMap<String, Vec<u8>>) {
    let mut producer = StoreHandle::open(local_config(&root.join("producer"))).await.unwrap();
    let info = signed_file(&mut producer, root, "real-signed-source", 71).await;
    let cache = root.join("cache");
    let report =
        crunch_store::export_paths_to_cache_dir(&producer, std::slice::from_ref(&info), &cache, &PushOptions {
            trust_unsigned: false,
        })
        .await
        .unwrap();
    assert_eq!(report.pushed_count, 1);
    let narinfo = format!("{}.narinfo", nix_compat::nixbase32::encode(info.store_path.digest()));
    let nar = format!("nar/{}.nar", nix_compat::nixbase32::encode(&info.nar_sha256));
    let files = ["nix-cache-info".to_string(), narinfo, nar]
        .into_iter()
        .map(|name| (format!("/{name}"), std::fs::read(cache.join(name)).unwrap()))
        .collect();
    (info, files)
}

#[derive(Default)]
struct Requests {
    total: AtomicUsize,
    metadata: AtomicUsize,
    narinfo: AtomicUsize,
    nar: AtomicUsize,
}

struct CacheServer {
    url: url::Url,
    requests: Arc<Requests>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl CacheServer {
    fn start(files: BTreeMap<String, Vec<u8>>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("source scenarios require loopback TCP");
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap()).parse().unwrap();
        let requests = Arc::new(Requests::default());
        let stop = Arc::new(AtomicBool::new(false));
        let counts = requests.clone();
        let stopped = stop.clone();
        let thread = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(30);
            loop {
                assert!(Instant::now() < deadline, "bounded source fixture listener expired");
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        counts.total.fetch_add(1, Ordering::SeqCst);
                        serve(&mut stream, &files, &counts);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        // Drain every pending connection before acknowledging stop.
                        if stopped.load(Ordering::SeqCst) {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(2));
                    }
                    Err(error) => panic!("fixture accept failed: {error}"),
                }
            }
        });
        Self {
            url,
            requests,
            stop,
            thread: Some(thread),
        }
    }

    fn finish(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

impl Drop for CacheServer {
    fn drop(&mut self) {
        self.finish();
    }
}

fn serve(stream: &mut TcpStream, files: &BTreeMap<String, Vec<u8>>, counts: &Requests) {
    stream.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
    stream.set_write_timeout(Some(Duration::from_secs(1))).unwrap();
    let mut request = Vec::new();
    let mut buffer = [0; 1024];
    while request.len() < 8192 {
        let read = stream.read(&mut buffer).unwrap();
        if read == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..read]);
        if request.windows(4).any(|part| part == b"\r\n\r\n") {
            break;
        }
    }
    let request = String::from_utf8(request).unwrap();
    let path = request.lines().next().unwrap().split_whitespace().nth(1).unwrap();
    match path {
        "/nix-cache-info" => {
            counts.metadata.fetch_add(1, Ordering::SeqCst);
        }
        path if path.ends_with(".narinfo") => {
            counts.narinfo.fetch_add(1, Ordering::SeqCst);
        }
        path if path.starts_with("/nar/") => {
            counts.nar.fetch_add(1, Ordering::SeqCst);
        }
        _ => {}
    }
    let (status, body) = match files.get(path) {
        Some(body) => ("200 OK", body.as_slice()),
        None => ("404 Not Found", &b""[..]),
    };
    write!(stream, "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
    stream.write_all(body).unwrap();
}

async fn assert_signed_content(store: &mut StoreHandle, expected: &PathInfo) {
    let actual = store.export_cached_path_info(&expected.store_path).await.unwrap().unwrap();
    assert_eq!(actual, *expected);
    let references: Vec<_> = actual.references.iter().map(StorePath::as_ref).collect();
    let fingerprint = fingerprint_with_store_dir(
        &actual.store_path.as_ref(),
        &actual.nar_sha256,
        actual.nar_size,
        references.iter(),
        PREFIX,
    );
    assert!(actual.signatures.iter().any(|signature| verifying_key().verify(&fingerprint, &signature.as_ref())));
    let output = Path::new(store.output_dir_str()).join(actual.store_path.to_string());
    assert_eq!(std::fs::read(&output).unwrap(), CONTENT);
    let mut nar = Vec::new();
    let (mut reader, writer) = tokio::io::duplex(65_536);
    let node = actual.node.clone();
    let blobs = store.blob_service();
    let directories = store.directory_service();
    let task = tokio::spawn(async move { snix_store::nar::write_nar(writer, &node, blobs, directories).await });
    tokio::io::AsyncReadExt::read_to_end(&mut reader, &mut nar).await.unwrap();
    task.await.unwrap().unwrap();
    assert_eq!(nar.len() as u64, actual.nar_size);
    assert_eq!(<[u8; 32]>::from(Sha256::digest(&nar)), actual.nar_sha256);
}

// r[impl remote_builds.single_binding_resolution]
#[tokio::test]
async fn configured_cache_denies_missing_forged_stale_revoked_and_expired_before_any_request() {
    let fixture = tempfile::tempdir().unwrap();
    let (_, files) = signed_cache(fixture.path()).await;
    let mut server = CacheServer::start(files);
    for case in [
        "missing", "forged", "stale", "revoked", "expired", "scope", "trust", "prefix", "source",
    ] {
        let root = tempfile::tempdir().unwrap();
        let authority = root.path().join("authority");
        let bindings = EndpointBindingStore::open(&authority).unwrap();
        let mut record = binding(&bindings, "cache-a", EndpointKind::Substituter);
        match case {
            "forged" => record.generation += 1,
            "stale" => {
                bindings.revoke(&record).unwrap();
                let next = bindings
                    .create(record.spec.clone(), Some(record.generation), crate::unix_time_now_s().unwrap())
                    .unwrap();
                assert_eq!(next.generation, record.generation + 1);
            }
            "revoked" => bindings.revoke(&record).unwrap(),
            "expired" => {
                bindings.revoke(&record).unwrap();
                let mut spec = record.spec.clone();
                spec.expires_unix_s = 2;
                record = bindings.create(spec, Some(record.generation), 1).unwrap();
            }
            _ => {}
        }
        let mut source_row = row(server.url.as_str(), &record);
        match case {
            "scope" => source_row["scope"] = json!("other-source-scope"),
            "trust" => source_row["trust_requirement"] = json!("other-source-key"),
            "prefix" => source_row["endpoint_binding"]["spec"]["store_prefix"] = json!("/other/store"),
            "source" => source_row["source"] = json!(server.url.join("different/").unwrap().as_str()),
            _ => {}
        }
        let config = load_config(root.path(), &authority, if case == "missing" { vec![] } else { vec![source_row] });
        let mut target = local_config(&root.path().join("target"));
        let state = target.state_dir.clone();
        let output = target.output_dir.clone();
        target.remote_cache_urls.push(server.url.to_string());
        target.source_admission = Some(Arc::new(config));
        assert!(StoreHandle::open(target).await.is_err(), "{case}");
        assert!(!state.exists(), "{case}: writable state was created");
        assert!(!output.exists(), "{case}: output directory was created");
    }
    server.finish();
    assert_eq!(server.requests.total.load(Ordering::SeqCst), 0);
    assert_eq!(server.requests.metadata.load(Ordering::SeqCst), 0);
    assert_eq!(server.requests.narinfo.load(Ordering::SeqCst), 0);
    assert_eq!(server.requests.nar.load(Ordering::SeqCst), 0);
}

// r[impl remote_builds.binding_revocation]
#[tokio::test]
async fn exact_cache_binding_pulls_signed_bytes_and_revocation_preserves_local_admission() {
    let root = tempfile::tempdir().unwrap();
    let (info, files) = signed_cache(root.path()).await;
    let mut server = CacheServer::start(files);
    let authority = root.path().join("authority");
    let bindings = EndpointBindingStore::open(&authority).unwrap();
    let record = binding(&bindings, "cache-a", EndpointKind::Substituter);
    let config = load_config(root.path(), &authority, vec![row(server.url.as_str(), &record)]);
    let target = root.path().join("target");
    let mut remote = local_config(&target);
    remote.remote_cache_urls.push(server.url.to_string());
    remote.source_admission = Some(Arc::new(config.clone()));
    let mut store = StoreHandle::open(remote).await.unwrap();
    let report = crunch_store::import_paths_from_http_cache(
        &store,
        &server.url,
        std::slice::from_ref(&info.store_path),
        &PullOptions {
            trust_unsigned: false,
            trusted_public_keys: vec![verifying_key()],
        },
    )
    .await
    .unwrap();
    assert_eq!(report.imported_count, 1);
    assert_eq!(report.skipped_untrusted_count, 0);
    assert_eq!(report.skipped_hash_mismatch_count, 0);
    assert_signed_content(&mut store, &info).await;
    drop(store); // The lifetime-held source lease must be released before revoke.
    let mut local = StoreHandle::open(local_config(&target)).await.unwrap();
    bindings.revoke(&record).unwrap();
    let before = server.requests.total.load(Ordering::SeqCst);
    let mut denied = local_config(&root.path().join("denied"));
    denied.remote_cache_urls.push(server.url.to_string());
    denied.source_admission = Some(Arc::new(config.clone()));
    assert!(StoreHandle::open(denied).await.is_err());
    // Even a local-only handle with an admission policy cannot bypass a revoked
    // source by invoking the public HTTP pull operation directly.
    let mut direct = local_config(&root.path().join("direct"));
    direct.source_admission = Some(Arc::new(config));
    let direct = StoreHandle::open(direct).await.unwrap();
    assert!(
        crunch_store::import_paths_from_http_cache(
            &direct,
            &server.url,
            std::slice::from_ref(&info.store_path),
            &PullOptions {
                trust_unsigned: false,
                trusted_public_keys: vec![verifying_key()]
            },
        )
        .await
        .is_err()
    );
    assert!(direct.pathinfo_service().get(*info.store_path.digest()).await.unwrap().is_none());
    assert_signed_content(&mut local, &info).await;
    server.finish();
    assert_eq!(server.requests.total.load(Ordering::SeqCst), before);
}

#[tokio::test]
async fn live_cache_binding_still_rejects_untrusted_signatures_and_corrupt_nar_content() {
    for corrupt in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let (info, mut files) = signed_cache(root.path()).await;
        if corrupt {
            let nar = files.iter_mut().find(|(name, _)| name.starts_with("/nar/")).unwrap().1;
            let position = nar.windows(CONTENT.len()).position(|part| part == CONTENT).unwrap();
            nar[position] ^= 1;
        }
        let mut server = CacheServer::start(files);
        let authority = root.path().join("authority");
        let bindings = EndpointBindingStore::open(&authority).unwrap();
        let record = binding(&bindings, "cache-a", EndpointKind::Substituter);
        let config = load_config(root.path(), &authority, vec![row(server.url.as_str(), &record)]);
        let mut target = local_config(&root.path().join("target"));
        target.remote_cache_urls.push(server.url.to_string());
        target.source_admission = Some(Arc::new(config));
        let store = StoreHandle::open(target).await.unwrap();
        let key = if corrupt {
            verifying_key()
        } else {
            let secret = ed25519_dalek::SigningKey::from_bytes(&[72; 32]);
            VerifyingKey::new("source-binding-fixture-1".into(), ed25519_dalek::VerifyingKey::from(&secret))
        };
        let report = crunch_store::import_paths_from_http_cache(
            &store,
            &server.url,
            std::slice::from_ref(&info.store_path),
            &PullOptions {
                trust_unsigned: false,
                trusted_public_keys: vec![key],
            },
        )
        .await
        .unwrap();
        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_untrusted_count, u32::from(!corrupt));
        assert_eq!(report.skipped_hash_mismatch_count, u32::from(corrupt));
        assert!(store.pathinfo_service().get(*info.store_path.digest()).await.unwrap().is_none());
        assert!(!Path::new(store.output_dir_str()).join(info.store_path.to_string()).exists());
        server.finish();
        assert_eq!(server.requests.narinfo.load(Ordering::SeqCst), 1);
        assert_eq!(server.requests.nar.load(Ordering::SeqCst), usize::from(corrupt));
    }
}

fn set_read_only(root: &Path, read_only: bool) {
    let mut directories = vec![root.to_path_buf()];
    let mut visited = Vec::new();
    while let Some(directory) = directories.pop() {
        if !read_only {
            std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        for entry in std::fs::read_dir(&directory).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                directories.push(entry.path());
            } else {
                std::fs::set_permissions(
                    entry.path(),
                    std::fs::Permissions::from_mode(if read_only { 0o400 } else { 0o600 }),
                )
                .unwrap();
            }
        }
        visited.push(directory);
    }
    for directory in visited.into_iter().rev() {
        std::fs::set_permissions(directory, std::fs::Permissions::from_mode(if read_only { 0o500 } else { 0o700 }))
            .unwrap();
    }
}

// r[impl remote_builds.single_binding_resolution]
// r[impl remote_builds.binding_revocation]
#[test]
fn shared_cache_and_base_leases_preserve_signed_content_generation_and_revocation_fences() {
    // A synchronous lock regression must fail within a bound, not hang the suite.
    let (done, completed) = std::sync::mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let result = std::panic::catch_unwind(|| {
            tokio::runtime::Runtime::new().unwrap().block_on(base_and_cache_scenario());
        });
        done.send(result).unwrap();
    });
    let result = completed.recv_timeout(Duration::from_secs(25)).expect("shared source lease acquisition deadlocked");
    worker.join().unwrap();
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}

async fn base_and_cache_scenario() {
    let root = tempfile::tempdir().unwrap();
    let (cache_info, files) = signed_cache(root.path()).await;
    let mut server = CacheServer::start(files);
    let base_root = root.path().join("base");
    let base_config = local_config(&base_root);
    let base_state = base_config.state_dir.clone();
    let mut producer = StoreHandle::open(base_config).await.unwrap();
    let base_info = signed_file(&mut producer, root.path(), "real-signed-base", 73).await;
    drop(producer);
    std::fs::write(base_state.join("overlay-trusted-public-keys"), format!("{}\n", verifying_key())).unwrap();
    set_read_only(&base_state, true);
    let canonical_base = std::fs::canonicalize(&base_state).unwrap();
    let authority = root.path().join("authority");
    let bindings = EndpointBindingStore::open(&authority).unwrap();
    let base_record = binding(&bindings, "base-a", EndpointKind::BaseLayer);
    let cache_record = binding(&bindings, "cache-a", EndpointKind::Substituter);
    let config = load_config(root.path(), &authority, vec![
        row(canonical_base.to_str().unwrap(), &base_record),
        row(server.url.as_str(), &cache_record),
    ]);
    let overlay_root = root.path().join("overlay");
    let mut target = local_config(&overlay_root).with_base_state_dirs(vec![canonical_base.clone()]);
    target.remote_cache_urls.push(server.url.to_string());
    target.source_admission = Some(Arc::new(config.clone()));
    std::fs::create_dir_all(&target.state_dir).unwrap();
    std::fs::write(target.state_dir.join("overlay-trusted-public-keys"), format!("{}\n", verifying_key())).unwrap();
    let mut overlay = StoreHandle::open(target).await.unwrap();
    let selected = overlay.path_info_with_layer(&base_info.store_path).await.unwrap().unwrap();
    assert_eq!(selected.layer, crunch_store::layer::StoreLayer::Base { index: 1 });
    assert_eq!(selected.value, base_info);
    assert_signed_content(&mut overlay, &base_info).await;
    let generation = overlay.overlay_report().unwrap().unwrap().bases[0].generation_blake3.clone();
    let report = crunch_store::import_paths_from_http_cache(
        &overlay,
        &server.url,
        std::slice::from_ref(&cache_info.store_path),
        &PullOptions {
            trust_unsigned: false,
            trusted_public_keys: vec![verifying_key()],
        },
    )
    .await
    .unwrap();
    assert_eq!(report.imported_count, 1);
    assert_eq!(overlay.overlay_report().unwrap().unwrap().bases[0].generation_blake3, generation);
    assert_signed_content(&mut overlay, &cache_info).await;
    // A genuine admitted layer remains subject to its independent generation fence.
    set_read_only(&base_state, false);
    std::fs::write(base_state.join("generation-drift"), b"changed after admission").unwrap();
    set_read_only(&base_state, true);
    assert!(overlay.revalidate_overlay_bases().is_err());
    drop(overlay);
    let mut local = StoreHandle::open(local_config(&overlay_root)).await.unwrap();
    assert!(
        local.pathinfo_service().get(*base_info.store_path.digest()).await.unwrap().is_none(),
        "base lookup backfilled the writable overlay"
    );
    bindings.revoke(&base_record).unwrap();
    // Malformed descriptor data would fail if inspected. Revocation must win first.
    set_read_only(&base_state, false);
    std::fs::write(base_state.join("store-identity.json"), b"not a layer descriptor").unwrap();
    set_read_only(&base_state, true);
    let expected_denial = config.admit(StoreSource::BaseLayer(&canonical_base), PREFIX).unwrap_err();
    let mut denied = local_config(&root.path().join("denied-base")).with_base_state_dirs(vec![canonical_base]);
    let denied_state = denied.state_dir.clone();
    denied.source_admission = Some(Arc::new(config));
    let error = StoreHandle::open(denied).await.err().expect("revoked base must not reopen");
    assert_eq!(error.to_string(), expected_denial.to_string(), "descriptor read preceded revoked-name rejection");
    assert!(!denied_state.exists());
    assert_signed_content(&mut local, &cache_info).await;
    server.finish();
    assert_eq!(server.requests.nar.load(Ordering::SeqCst), 1);
    set_read_only(&base_state, false);
}
