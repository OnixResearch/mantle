//! Shared test helpers for crunch-build tests.
//!
//! Provides MockBuildService, build_and_register, and factory functions
//! for in-memory blob/directory/pathinfo services.
#![cfg(test)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::Mutex;

use async_trait::async_trait;
use nix_compat::derivation::Derivation;
use nix_compat::derivation::Output;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::store_path::StorePath;
use snix_build::buildservice::BuildOutput;
use snix_build::buildservice::BuildRequest;
use snix_build::buildservice::BuildResult;
use snix_build::buildservice::BuildService;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::blobservice::MemoryBlobService;
use snix_castore::directoryservice::RedbDirectoryService;
use snix_castore::directoryservice::RedbDirectoryServiceConfig;
use snix_store::pathinfoservice::LruPathInfoService;
use tokio::io::AsyncWriteExt;

use crate::registry::DerivationRegistry;
use crate::signing;

/// Create a temporary in-memory directory service.
pub fn tmp_ds() -> RedbDirectoryService {
    RedbDirectoryService::new_temporary("test".to_string(), RedbDirectoryServiceConfig::default()).unwrap()
}

/// A test keypair (same as nix-compat's DUMMY_KEYPAIR).
pub fn test_keypair() -> signing::KeyPair {
    signing::load_keypair(
        "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==",
    )
    .unwrap()
}

/// Default trusted keys for tests (local + cache.nixos.org-1).
pub fn test_trusted_keys() -> Vec<VerifyingKey> {
    signing::build_trusted_keys(&test_keypair(), None)
}

/// Create an in-memory LRU path info service.
pub fn test_pis() -> LruPathInfoService {
    LruPathInfoService::with_capacity("test".to_string(), std::num::NonZeroUsize::new(128).unwrap())
}

/// A mock BuildService that records command_args from each request and
/// returns a synthetic output node for each requested output.
pub struct MockBuildService {
    calls: Arc<Mutex<Vec<Vec<String>>>>,
    blob_service: MemoryBlobService,
}

impl MockBuildService {
    pub fn new(blob_service: MemoryBlobService) -> (Self, Arc<Mutex<Vec<Vec<String>>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                calls: calls.clone(),
                blob_service,
            },
            calls,
        )
    }
}

#[async_trait]
#[async_trait]
impl BuildService for MockBuildService {
    async fn do_build(&self, request: BuildRequest) -> std::io::Result<BuildResult> {
        self.calls.lock().unwrap().push(request.command_args.clone());

        let mut writer = BlobService::open_write(&self.blob_service).await;
        writer.write_all(b"mock output").await.unwrap();
        let digest = writer.close().await.unwrap();

        let outputs: Vec<BuildOutput> = request
            .outputs
            .iter()
            .map(|_| BuildOutput {
                node: Node::File {
                    digest: digest.clone(),
                    size: 11,
                    executable: false,
                },
                output_needles: BTreeSet::new(),
            })
            .collect();

        Ok(BuildResult { outputs, log: None })
    }
}

/// Build a nix_compat::Derivation, compute paths, register in DerivationRegistry.
///
/// `input_drvs`: `&[(parent_store_path, output_name)]` — parents must
/// already be registered in `kp`.
pub fn build_and_register(
    name: &str,
    input_drvs: &[(StorePath<String>, &str)],
    kp: &mut DerivationRegistry,
) -> (StorePath<String>, Derivation) {
    let mut outputs = BTreeMap::new();
    outputs.insert("out".to_string(), Output {
        path: None,
        ca_hash: None,
    });
    let mut environment = BTreeMap::new();
    environment.insert("name".to_string(), name.into());
    environment.insert("system".to_string(), "x86_64-linux".into());
    environment.insert("builder".to_string(), "/bin/sh".into());
    environment.insert("out".to_string(), "".into());

    let mut input_derivations = BTreeMap::new();
    for (dp, on) in input_drvs {
        input_derivations.entry(dp.clone()).or_insert_with(BTreeSet::new).insert(on.to_string());
    }

    let mut drv = Derivation {
        arguments: vec!["-c".into(), format!("echo {name} > $out")],
        builder: "/bin/sh".to_string(),
        environment,
        input_derivations,
        input_sources: BTreeSet::new(),
        outputs,
        system: "x86_64-linux".to_string(),
    };

    let hdm = drv.hash_derivation_modulo(|parent_path| {
        kp.get_hdm_by_drv_path(&parent_path.to_absolute_path()).expect("parent should be in known_paths")
    });
    drv.calculate_output_paths(name, &hdm).unwrap();
    let drv_path = drv.calculate_derivation_path(name).unwrap();

    let mut fake_hash = [0u8; 32];
    for (i, b) in name.bytes().enumerate().take(32) {
        fake_hash[i] = b;
    }
    kp.insert(drv_path.clone(), hdm, drv.clone(), false);

    (drv_path, drv)
}

/// Like `build_and_register` but with multiple outputs.
///
/// Creates a derivation with the given output names (e.g., `["out", "dev", "lib"]`).
/// Each output gets a distinct store path.
pub fn build_and_register_multi(
    name: &str,
    output_names: &[&str],
    input_drvs: &[(StorePath<String>, &str)],
    kp: &mut DerivationRegistry,
) -> (StorePath<String>, Derivation) {
    assert!(!output_names.is_empty(), "must have at least one output");

    let mut outputs = BTreeMap::new();
    let mut environment = BTreeMap::new();
    environment.insert("name".to_string(), name.into());
    environment.insert("system".to_string(), "x86_64-linux".into());
    environment.insert("builder".to_string(), "/bin/sh".into());
    environment.insert("outputs".to_string(), output_names.join(" ").into());

    for on in output_names {
        outputs.insert(on.to_string(), Output {
            path: None,
            ca_hash: None,
        });
        environment.insert(on.to_string(), "".into());
    }

    let mut input_derivations = BTreeMap::new();
    for (dp, on) in input_drvs {
        input_derivations.entry(dp.clone()).or_insert_with(BTreeSet::new).insert(on.to_string());
    }

    let mut drv = Derivation {
        arguments: vec!["-c".into(), format!("echo {name} > $out")],
        builder: "/bin/sh".to_string(),
        environment,
        input_derivations,
        input_sources: BTreeSet::new(),
        outputs,
        system: "x86_64-linux".to_string(),
    };

    let hdm = drv.hash_derivation_modulo(|parent_path| {
        kp.get_hdm_by_drv_path(&parent_path.to_absolute_path()).expect("parent should be in known_paths")
    });
    drv.calculate_output_paths(name, &hdm).unwrap();
    let drv_path = drv.calculate_derivation_path(name).unwrap();

    let mut fake_hash = [0u8; 32];
    for (i, b) in name.bytes().enumerate().take(32) {
        fake_hash[i] = b;
    }
    kp.insert(drv_path.clone(), hdm, drv.clone(), false);

    (drv_path, drv)
}

/// Like `build_and_register` but names the output with `.drv` suffix
/// so the dynamic derivation detector picks it up.
#[allow(dead_code)]
pub fn build_and_register_producer(name: &str, kp: &mut DerivationRegistry) -> (StorePath<String>, Derivation) {
    // Producer name ends with .drv so output path triggers detection.
    let producer_name = format!("{name}.drv");
    build_and_register(&producer_name, &[], kp)
}

/// A mock BuildService that produces `.drv` ATerm content for builds
/// whose name contains a specific marker. All other builds get normal
/// mock output.
///
/// The "inner" derivation ATerm bytes are provided at construction.
pub struct DrvProducingMockBuildService {
    calls: Arc<Mutex<Vec<Vec<String>>>>,
    blob_service: MemoryBlobService,
    /// Map from name substring → ATerm bytes to produce as output.
    drv_outputs: HashMap<String, Vec<u8>>,
}

use std::collections::HashMap;

impl DrvProducingMockBuildService {
    pub fn new(
        blob_service: MemoryBlobService,
        drv_outputs: HashMap<String, Vec<u8>>,
    ) -> (Self, Arc<Mutex<Vec<Vec<String>>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                calls: calls.clone(),
                blob_service,
                drv_outputs,
            },
            calls,
        )
    }
}

#[async_trait]
#[async_trait]
impl BuildService for DrvProducingMockBuildService {
    async fn do_build(&self, request: BuildRequest) -> std::io::Result<BuildResult> {
        self.calls.lock().unwrap().push(request.command_args.clone());

        // Check if any name substring matches this request's args.
        let content: Vec<u8> = self
            .drv_outputs
            .iter()
            .find(|(marker, _)| request.command_args.iter().any(|a| a.contains(marker.as_str())))
            .map(|(_, aterm)| aterm.clone())
            .unwrap_or_else(|| b"mock output".to_vec());

        let size = content.len() as u64;
        let mut writer = BlobService::open_write(&self.blob_service).await;
        writer.write_all(&content).await.unwrap();
        let digest = writer.close().await.unwrap();

        let outputs: Vec<BuildOutput> = request
            .outputs
            .iter()
            .map(|_| BuildOutput {
                node: Node::File {
                    digest: digest.clone(),
                    size,
                    executable: false,
                },
                output_needles: BTreeSet::new(),
            })
            .collect();

        Ok(BuildResult { outputs, log: None })
    }
}

/// A mock BuildService that fails for builds whose command_args
/// contain a specific substring. All other builds succeed normally.
pub struct FailingMockBuildService {
    calls: Arc<Mutex<Vec<Vec<String>>>>,
    blob_service: MemoryBlobService,
    /// Substrings that trigger a build failure.
    fail_markers: Vec<String>,
}

impl FailingMockBuildService {
    pub fn new(blob_service: MemoryBlobService, fail_markers: Vec<String>) -> (Self, Arc<Mutex<Vec<Vec<String>>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                calls: calls.clone(),
                blob_service,
                fail_markers,
            },
            calls,
        )
    }
}

#[async_trait]
#[async_trait]
impl BuildService for FailingMockBuildService {
    async fn do_build(&self, request: BuildRequest) -> std::io::Result<BuildResult> {
        self.calls.lock().unwrap().push(request.command_args.clone());

        // Check if this build should fail.
        let should_fail = self
            .fail_markers
            .iter()
            .any(|marker| request.command_args.iter().any(|a| a.contains(marker.as_str())));

        if should_fail {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "simulated build failure"));
        }

        let mut writer = BlobService::open_write(&self.blob_service).await;
        writer.write_all(b"mock output").await.unwrap();
        let digest = writer.close().await.unwrap();

        let outputs: Vec<BuildOutput> = request
            .outputs
            .iter()
            .map(|_| BuildOutput {
                node: Node::File {
                    digest: digest.clone(),
                    size: 11,
                    executable: false,
                },
                output_needles: BTreeSet::new(),
            })
            .collect();

        Ok(BuildResult { outputs, log: None })
    }
}
